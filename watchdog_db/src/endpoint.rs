use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;
use watchdog_core::{Endpoint, EndpointRepo, RepoError};

pub struct PgEndpointRepo {
    pool: PgPool,
}

impl PgEndpointRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EndpointRow {
    id: Uuid,
    url: String,
    interval_seconds: i32,
    timeout: Option<i32>,
    expected_status: i32,
}

impl From<EndpointRow> for Endpoint {
    fn from(value: EndpointRow) -> Self {
        Endpoint {
            id: value.id,
            url: value.url,
            interval: value.interval_seconds as u32,
            timeout: value.timeout.map(|secs| Duration::from_secs(secs as u64)),
            expected_status: value.expected_status as u16,
        }
    }
}

impl EndpointRepo for PgEndpointRepo {
    fn create(
        &self,
        new_endpoint: watchdog_core::NewEndpoint,
    ) -> impl Future<Output = Result<watchdog_core::Endpoint, watchdog_core::RepoError>> + Send
    {
        async move {
            Ok(sqlx::query_as!(
                EndpointRow,
                r#"
                INSERT INTO endpoints (url, interval_seconds, timeout, expected_status)
                VALUES ($1, $2, $3, $4)
                RETURNING id, url, interval_seconds, timeout, expected_status
            "#,
                new_endpoint.url,
                new_endpoint.interval as i32,
                new_endpoint.timeout.map(|d| d.as_secs() as i32),
                new_endpoint.expected_status as i32,
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepoError::Database(err.to_string()))?
            .into())
        }
    }

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Endpoint>, RepoError>> + Send {
        async move {
            Ok(sqlx::query_as!(
                EndpointRow,
                r#"
                    SELECT id, url, interval_seconds, timeout, expected_status FROM endpoints WHERE id = $1
                "#,
                id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepoError::Database(err.to_string()))?
            .map(|endpoint_row| endpoint_row.into()))
        }
    }

    fn list(
        &self,
        limit: u16,
        offset: u16,
    ) -> impl Future<Output = Result<Vec<Endpoint>, RepoError>> + Send {
        async move {
            Ok(sqlx::query_as!(
                EndpointRow,
                r#"
                SELECT id, url, interval_seconds, timeout, expected_status FROM endpoints LIMIT $1 OFFSET $2
            "#,
                limit as i32,
                offset as i32,
            ).fetch_all(&self.pool)
            .await
            .map_err(|err| RepoError::Database(err.to_string()))?
            .into_iter().map(|endpoint_row| endpoint_row.into()).collect())
        }
    }

    fn delete(&self, endpoint_id: Uuid) -> impl Future<Output = Result<(), RepoError>> + Send {
        async move {
            sqlx::query!(
                r#"
                DELETE FROM endpoints WHERE id = $1
            "#,
                endpoint_id
            )
            .execute(&self.pool)
            .await
            .map_err(|err| RepoError::Database(err.to_string()))?;

            Ok(())
        }
    }
}
