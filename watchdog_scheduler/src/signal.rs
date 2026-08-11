use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::TcpListener, sync::RwLock};
use watchdog_core::{Endpoint, EndpointRepo};

pub async fn dispatch<E>(endpoint_repo: &Arc<E>, endpoints: &Arc<RwLock<Vec<Endpoint>>>)
where
    E: EndpointRepo + Send + Sync + 'static,
{
    let listener = match TcpListener::bind("0.0.0.0:8080").await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("{e}");
            return;
        }
    };

    loop {
        let (mut socket, _) = match listener.accept().await {
            Ok(accept) => accept,
            Err(e) => {
                tracing::error!("{e}");
                return;
            }
        };
        let endpoint_repo = endpoint_repo.clone();
        let endpoints = endpoints.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];

            if let Ok(n) = socket.read(&mut buf).await {
                if n > 0 {
                    let new_endpoints = match endpoint_repo.list(u16::MAX, 0).await {
                        Ok(endpoint) => endpoint,
                        Err(e) => {
                            tracing::error!("{e}");
                            return;
                        }
                    };

                    let mut lock = endpoints.write().await;
                    *lock = new_endpoints;
                }
            }
        });
    }
}
