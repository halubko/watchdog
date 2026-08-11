use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::task::JoinHandle;
use uuid::Uuid;
use watchdog_core::{CheckResultRepo, EndpointRepo, NewCheckResult};

use crate::checker::check;

pub mod checker;
pub mod signal;

pub async fn supervisor<E, C>(
    endpoints_repo: Arc<E>,
    check_result_repo: Arc<C>,
    client: Arc<reqwest::Client>,
) where
    E: EndpointRepo + Send + Sync + 'static,
    C: CheckResultRepo + Send + Sync + 'static,
{
    let mut active: HashMap<Uuid, JoinHandle<()>> = HashMap::new();
    let notify = Arc::new(tokio::sync::Notify::new());

    tokio::spawn(signal::dispatch(Arc::clone(&notify)));

    loop {
        let current_endpoints = match endpoints_repo.list(u16::MAX, 0).await {
            Ok(vec) => vec,
            Err(e) => {
                tracing::error!("{e}");
                return;
            }
        };

        for endpoint in &current_endpoints {
            if !active.contains_key(&endpoint.id) {
                let check_result_repo = Arc::clone(&check_result_repo);
                let client = Arc::clone(&client);
                let endpoint = endpoint.clone();
                let endpoint_id = endpoint.id;

                let handle = tokio::spawn(async move {
                    let mut interval =
                        tokio::time::interval(Duration::from_secs(endpoint.interval as u64));

                    loop {
                        interval.tick().await;

                        let status = check(&client, &endpoint).await;

                        let new_check_result = NewCheckResult {
                            endpoint_id: endpoint.id,
                            status,
                        };

                        match check_result_repo.save(new_check_result).await {
                            Ok(_) => tracing::info!("Check for {} saved", &endpoint.url),
                            Err(e) => tracing::error!("failed to save check result: {e}"),
                        }
                    }
                });

                active.insert(endpoint_id, handle);
            }
        }

        notify.notified().await;
    }
}
