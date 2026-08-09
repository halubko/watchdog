use std::{process, sync::Arc};

use watchdog_db::{check_result::PgCheckResultRepo, endpoint::PgEndpointRepo, migrate};
use watchdog_scheduler::run;

use crate::app::AppState;

pub mod app;
pub mod dto;
pub mod error;
pub mod handlers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = watchdog_config::config().unwrap_or_else(|err| {
        tracing::error!("{err}");
        process::exit(1);
    });

    tracing::info!("connecting to: {}", config.db_url());

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

    let client = reqwest::Client::new();
    let endpoint_repo = PgEndpointRepo::new(pool.clone());
    let check_results_repo = PgCheckResultRepo::new(pool);

    let state = Arc::new(AppState::new(
        endpoint_repo.clone(),
        check_results_repo.clone(),
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

    if let (Err(e), ()) = tokio::join!(
        axum::serve(listener, app),
        run(endpoint_repo, check_results_repo, client)
    ) {
        tracing::error!("{e}");
        process::exit(1)
    };
}
