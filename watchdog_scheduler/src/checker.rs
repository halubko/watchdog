use watchdog_core::{CheckStatus, Endpoint, FailureReason};

pub async fn check(client: &reqwest::Client, endpoint: &Endpoint) -> CheckStatus {
    let latency_ms = std::time::Instant::now();

    let mut request = client.get(&endpoint.url);

    if let Some(timeout) = endpoint.timeout {
        request = request.timeout(timeout);
    }

    let response = request.send().await;

    let latency_ms = latency_ms.elapsed().as_millis() as u64;

    match response {
        Ok(response) => {
            let status_code = response.status().as_u16();
            if status_code == endpoint.expected_status {
                CheckStatus::Success {
                    status_code,
                    latency_ms,
                }
            } else {
                CheckStatus::UnexpectedStatus {
                    status_code,
                    latency_ms,
                }
            }
        }
        Err(e) => {
            if e.is_timeout() {
                return CheckStatus::Fail {
                    reason: FailureReason::Timeout,
                };
            }
            if e.is_connect() {
                return CheckStatus::Fail {
                    reason: FailureReason::ConnectionRefused,
                };
            }
            CheckStatus::Fail {
                reason: FailureReason::Other(e.to_string()),
            }
        }
    }
}
