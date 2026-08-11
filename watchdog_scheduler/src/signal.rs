use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::TcpListener, sync};

pub async fn dispatch(notify: Arc<sync::Notify>) {
    let listener = match TcpListener::bind("localhost:8080").await {
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
        let notify = Arc::clone(&notify);

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];

            if let Ok(n) = socket.read(&mut buf).await {
                if n > 0 {
                    notify.notify_one();
                }
            }
        });
    }
}
