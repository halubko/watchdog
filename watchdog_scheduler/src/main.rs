use std::{process, sync::Arc};

use watchdog_db::{check_result::PgCheckResultRepo, endpoint::PgEndpointRepo};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_file(true).init();

    let client = Arc::new(reqwest::Client::new());

    let db_config = watchdog_config::db_config().unwrap_or_else(|err| {
        tracing::error!("{err}");
        process::exit(1);
    });

    let pool = match watchdog_db::connect(&db_config).await {
        Ok(connection) => {
            tracing::info!("DB connected successfully");
            connection
        }
        Err(err) => {
            tracing::error!("{err}");
            process::exit(1)
        }
    };

    let endpoint_repo = Arc::new(PgEndpointRepo::new(pool.clone()));
    let check_results_repo = Arc::new(PgCheckResultRepo::new(pool));

    watchdog_scheduler::supervisor(endpoint_repo, check_results_repo, client).await;
}
