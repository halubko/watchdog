use sqlx::PgPool;
use watchdog_core::{CheckResult, CheckResultRepo, CheckStatus, FailureReason, RepoError};

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct PgCheckResultRepo {
    pool: PgPool,
}

impl PgCheckResultRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CheckResultRow {
    id: Uuid,
    endpoint_id: Uuid,
    date: DateTime<Utc>,
    status: CheckResultRowStatus,
    status_code: Option<i32>,
    latency_ms: Option<i32>,
    failure_reason: Option<Reason>,
    failure_message: Option<String>,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "check_status", rename_all = "snake_case")]
pub enum CheckResultRowStatus {
    Success,
    UnexpectedStatus,
    Fail,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "failure_reason", rename_all = "snake_case")]
enum Reason {
    Timeout,
    ConnectionRefused,
    DnsResolution,
    Other,
}

#[derive(Debug, thiserror::Error)]
enum CheckResultConvertError {
    #[error("row has status 'success' but status_code or latency_ms is missing")]
    MissingSuccessFields,
    #[error("row has status 'unexpected_status' but status_code or latency_ms is missing")]
    MissingUnexpectedSuccessFields,
    #[error("row has status 'fail' but failure_reason is missing")]
    MissingFailureReason,
}

impl TryFrom<CheckResultRow> for CheckResult {
    type Error = CheckResultConvertError;

    fn try_from(value: CheckResultRow) -> Result<Self, Self::Error> {
        let status = match value.status {
            CheckResultRowStatus::Success => CheckStatus::Success {
                status_code: value
                    .status_code
                    .ok_or(CheckResultConvertError::MissingSuccessFields)?
                    as u16,
                latency_ms: value
                    .latency_ms
                    .ok_or(CheckResultConvertError::MissingSuccessFields)?
                    as u64,
            },
            CheckResultRowStatus::UnexpectedStatus => CheckStatus::UnexpectedStatus {
                status_code: value
                    .status_code
                    .ok_or(CheckResultConvertError::MissingUnexpectedSuccessFields)?
                    as u16,
                latency_ms: value
                    .latency_ms
                    .ok_or(CheckResultConvertError::MissingUnexpectedSuccessFields)?
                    as u64,
            },
            CheckResultRowStatus::Fail => CheckStatus::Fail {
                reason: match value
                    .failure_reason
                    .ok_or(CheckResultConvertError::MissingFailureReason)?
                {
                    Reason::Timeout => FailureReason::Timeout,
                    Reason::ConnectionRefused => FailureReason::ConnectionRefused,
                    Reason::DnsResolution => FailureReason::DnsResolution,
                    Reason::Other => FailureReason::Other(
                        value
                            .failure_message
                            .ok_or(CheckResultConvertError::MissingFailureReason)?,
                    ),
                },
            },
        };

        Ok(CheckResult {
            id: value.id,
            endpoint_id: value.endpoint_id,
            date: value.date,
            status,
        })
    }
}

impl CheckResultRepo for PgCheckResultRepo {
    fn save(
        &self,
        new_check_result: watchdog_core::NewCheckResult,
    ) -> impl Future<Output = Result<CheckResult, watchdog_core::RepoError>> + Send {
        let (status, status_code, latency_ms, failure_reason, failure_message) =
            match new_check_result.status {
                CheckStatus::Success {
                    status_code,
                    latency_ms,
                } => (
                    CheckResultRowStatus::Success,
                    Some(status_code as i32),
                    Some(latency_ms as i32),
                    None,
                    None,
                ),
                CheckStatus::UnexpectedStatus {
                    status_code,
                    latency_ms,
                } => (
                    CheckResultRowStatus::UnexpectedStatus,
                    Some(status_code as i32),
                    Some(latency_ms as i32),
                    None,
                    None,
                ),
                CheckStatus::Fail { reason } => {
                    let (reason_kind, message) = match reason {
                        FailureReason::ConnectionRefused => (Reason::ConnectionRefused, None),
                        FailureReason::Timeout => (Reason::Timeout, None),
                        FailureReason::DnsResolution => (Reason::DnsResolution, None),
                        FailureReason::Other(msg) => (Reason::Other, Some(msg)),
                    };
                    (
                        CheckResultRowStatus::Fail,
                        None,
                        None,
                        Some(reason_kind),
                        message,
                    )
                }
            };

        async move {
            sqlx::query_as!(
                CheckResultRow,
                r#"
                    INSERT INTO check_results (endpoint_id, status_code, latency_ms, failure_reason, failure_message, status)
                    VALUES($1, $2, $3, $4, $5, $6)
                    RETURNING id, endpoint_id, date, status_code, latency_ms,  failure_reason as "failure_reason: Reason", failure_message, status as "status: CheckResultRowStatus"
                "#,
                new_check_result.endpoint_id,
                status_code,
                latency_ms,
                failure_reason as Option<Reason>,
                failure_message,
                status as CheckResultRowStatus
            ).fetch_one(&self.pool).await
            .map_err(|err| RepoError::Database( err.to_string()))?
            .try_into().map_err(|err: CheckResultConvertError| RepoError::Database(err.to_string()))
        }
    }

    fn list(
        &self,
        endpoint_id: Uuid,
        limit: u16,
        offset: u16,
    ) -> impl Future<Output = Result<Vec<CheckResult>, RepoError>> + Send {
        async move {
            sqlx::query_as!(
                CheckResultRow,
                r#"
                    SELECT id, endpoint_id, date, status_code, latency_ms, failure_reason as "failure_reason: Reason", failure_message, status as "status: CheckResultRowStatus" FROM check_results
                    WHERE endpoint_id = $1
                    LIMIT $2
                    OFFSET $3
                "#,
                endpoint_id,
                limit as i32,
                offset as i32
            ).fetch_all(&self.pool).await
            .map_err(|err| RepoError::Database(err.to_string()))?
            .into_iter().map(|check_result_row| check_result_row.try_into().map_err(|err: CheckResultConvertError| RepoError::Database(err.to_string()))).collect()
        }
    }

    fn uptime_percentage(
        &self,
        endpoint_id: Uuid,
        since: DateTime<Utc>,
    ) -> impl Future<Output = Result<f64, RepoError>> + Send {
        async move {
            Ok(sqlx::query!(
                r#"
                    SELECT (COUNT(*) FILTER (WHERE status = 'success')::float8 * 100.0 / NULLIF(COUNT(*), 0)::float8) AS uptime_percentage
                    FROM check_results
                    WHERE endpoint_id = $1 AND date >= $2
                "#,
                endpoint_id,
                since
            ).fetch_one(&self.pool).await
            .map_err(|err| RepoError::Database(err.to_string()))?
            .uptime_percentage.unwrap_or(0.0))
        }
    }
}
