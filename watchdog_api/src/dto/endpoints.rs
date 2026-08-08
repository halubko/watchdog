use std::time::Duration;

use uuid::Uuid;
use watchdog_core::{Endpoint, NewEndpoint};

#[derive(serde::Deserialize)]
pub struct CreateEndpointRequest {
    pub url: String,
    pub interval_seconds: u32,
    pub timeout_seconds: Option<u64>,
    pub expected_status: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("url must not be empty")]
    EmptyUrl,
    #[error("interval_seconds must be greater than 0")]
    ZeroInterval,
}

impl TryFrom<CreateEndpointRequest> for NewEndpoint {
    type Error = ValidationError;

    fn try_from(value: CreateEndpointRequest) -> Result<Self, Self::Error> {
        if value.interval_seconds == 0 {
            return Err(ValidationError::ZeroInterval);
        }

        if value.url.trim().is_empty() {
            return Err(ValidationError::EmptyUrl);
        }

        Ok(NewEndpoint {
            url: value.url,
            interval: value.interval_seconds,
            timeout: value.timeout_seconds.map(Duration::from_secs),
            expected_status: value.expected_status,
        })
    }
}

#[derive(serde::Serialize)]
pub struct EndpointResponse {
    id: Uuid,
    url: String,
    interval_seconds: u32,
    timeout_seconds: Option<u64>,
    expected_status: u16,
}

impl From<Endpoint> for EndpointResponse {
    fn from(value: Endpoint) -> Self {
        EndpointResponse {
            id: value.id,
            url: value.url,
            interval_seconds: value.interval,
            timeout_seconds: value.timeout.map(|secs| secs.as_secs()),
            expected_status: value.expected_status,
        }
    }
}
