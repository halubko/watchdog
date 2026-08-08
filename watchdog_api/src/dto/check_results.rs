use chrono::{DateTime, Utc};
use uuid::Uuid;
use watchdog_core::{CheckResult, CheckStatus, FailureReason};

#[derive(serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CheckResultResponseStatuses {
    Success {
        status_code: u16,
        latency_ms: u64,
    },
    UnexpectedStatus {
        status_code: u16,
        latency_ms: u64,
    },
    Fail {
        failure_reason: FailureReasonResponse,
        failure_message: Option<String>,
    },
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureReasonResponse {
    Timeout,
    ConnectionRefused,
    DnsResolution,
    Other,
}

impl From<FailureReason> for FailureReasonResponse {
    fn from(value: FailureReason) -> Self {
        match value {
            FailureReason::ConnectionRefused => Self::ConnectionRefused,
            FailureReason::DnsResolution => Self::DnsResolution,
            FailureReason::Timeout => FailureReasonResponse::Timeout,
            FailureReason::Other(_) => Self::Other,
        }
    }
}

#[derive(serde::Serialize)]
pub struct CheckResultResponse {
    id: Uuid,
    endpoint_id: Uuid,
    status: CheckResultResponseStatuses,
}

impl From<CheckResult> for CheckResultResponse {
    fn from(value: CheckResult) -> CheckResultResponse {
        let status = match value.status {
            CheckStatus::Success {
                status_code,
                latency_ms,
            } => CheckResultResponseStatuses::Success {
                status_code,
                latency_ms,
            },
            CheckStatus::UnexpectedStatus {
                status_code,
                latency_ms,
            } => CheckResultResponseStatuses::UnexpectedStatus {
                status_code,
                latency_ms,
            },
            CheckStatus::Fail { reason } => {
                let failure_message = match &reason {
                    FailureReason::Other(message) => Some(message.clone()),
                    _ => None,
                };
                CheckResultResponseStatuses::Fail {
                    failure_reason: reason.into(),
                    failure_message,
                }
            }
        };

        CheckResultResponse {
            id: value.id,
            endpoint_id: value.endpoint_id,
            status,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct UptimePercentageRequest {
    pub since: DateTime<Utc>,
}

#[derive(serde::Serialize)]
pub struct UptimePercentageResponse {
    pub uptime_percentage: f64,
    pub since: DateTime<Utc>,
}
