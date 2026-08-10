use std::{sync::Arc, time::Duration};

use tokio::task::JoinHandle;
use watchdog_core::{CheckResultRepo, EndpointRepo, NewCheckResult};

use crate::checker::check;

pub mod checker;

pub async fn run<E, C>(endpoint_repo: E, check_result_repo: C, client: reqwest::Client)
where
    E: EndpointRepo + Send + Sync + 'static,
    C: CheckResultRepo + Send + Sync + 'static,
{
    let check_result_repo = Arc::new(check_result_repo);
    let client = Arc::new(client);

    let endpoints = match endpoint_repo.list(u16::MAX, 0).await {
        Ok(endpoint) => endpoint,
        Err(e) => {
            tracing::error!("{e}");
            return;
        }
    };

    let mut handlers: Vec<JoinHandle<()>> = vec![];

    for endpoint in endpoints {
        let check_result_repo = check_result_repo.clone();
        let client = Arc::clone(&client);

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(endpoint.interval as u64));

            loop {
                let status = check(&client, &endpoint).await;

                let new_check_result = NewCheckResult {
                    endpoint_id: endpoint.id,
                    status,
                };

                match check_result_repo.save(new_check_result).await {
                    Ok(_) => tracing::info!("Check for {} saved", &endpoint.url),
                    Err(e) => tracing::error!("failed to save check result: {e}"),
                };

                interval.tick().await;
            }
        });

        handlers.push(task);
    }

    for handler in handlers {
        let _ = handler.await;
    }
}
