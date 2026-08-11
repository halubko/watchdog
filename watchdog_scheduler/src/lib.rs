use std::{sync::Arc, time::Duration};

use tokio::{sync::RwLock, task::JoinHandle};
use watchdog_core::{CheckResultRepo, Endpoint, EndpointRepo, NewCheckResult};

use crate::{checker::check, signal::dispatch};

pub mod checker;
pub mod signal;

pub struct EndpointCache {
    pub endpoint_repo: Arc<dyn EndpointRepo + Send + Sync>,
    pub endpoints: Arc<RwLock<Vec<Endpoint>>>,
}

pub async fn run<E, C>(endpoint_repo: E, check_result_repo: C, client: reqwest::Client)
where
    E: EndpointRepo + Send + Sync + 'static,
    C: CheckResultRepo + Send + Sync + 'static,
{
    let endpoint_repo = Arc::new(endpoint_repo);
    let check_result_repo = Arc::new(check_result_repo);
    let client = Arc::new(client);

    let endpoints: Arc<RwLock<Vec<Endpoint>>> = Arc::new(RwLock::new(vec![]));

    dispatch(&endpoint_repo, &endpoints).await;

    let mut handlers: Vec<JoinHandle<()>> = vec![];

    let endpoints_vec = endpoints.clone().read().await.to_vec();

    for endpoint in endpoints_vec {
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
