use std::{process, sync::Arc};

use watchdog_db::{check_result::PgCheckResultRepo, endpoint::PgEndpointRepo, migrate};

use crate::{app::AppState, notifier::Notifier};

pub mod app;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod notifier;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_file(true).init();

    let config = watchdog_config::config().unwrap_or_else(|err| {
        tracing::error!("{err}");
        process::exit(1);
    });

    let pool = match watchdog_db::connect(&config.database).await {
        Ok(connection) => {
            tracing::info!("DB connected successfully");
            connection
        }
        Err(err) => {
            tracing::error!("{err}");
            process::exit(1)
        }
    };

    if let Err(e) = migrate(&pool).await {
        tracing::error!("{e}");
        process::exit(1)
    }

    let endpoint_repo = PgEndpointRepo::new(pool.clone());
    let check_results_repo = PgCheckResultRepo::new(pool);
    let notifier = Arc::new(Notifier::new("scheduler:8080".to_string()));

    let state = Arc::new(AppState::new(
        endpoint_repo.clone(),
        check_results_repo.clone(),
        notifier,
    ));

    let app = app::app(state);

    let listener = match tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.server.host, config.server.port
    ))
    .await
    {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!("{err}");
            process::exit(1)
        }
    };

    tracing::info!("listening on {}:{}", config.server.host, config.server.port);

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("{e}");
        process::exit(1)
    };
}
