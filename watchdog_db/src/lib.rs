use sqlx::{Error, PgPool, postgres::PgPoolOptions};
use watchdog_config::DatabaseConfig;

pub mod check_result;
pub mod endpoint;

pub async fn connect(db_conf: &DatabaseConfig) -> Result<PgPool, Error> {
    PgPoolOptions::new()
        .max_connections(db_conf.max_connections)
        .connect(&db_conf.url())
        .await
}
