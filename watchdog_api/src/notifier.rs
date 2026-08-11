use std::time::Duration;

use tokio::{io::AsyncWriteExt, net::TcpStream, time::sleep};

pub struct Notifier {
    addr: String,
}

impl Notifier {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn notify(&self) {
        for attempt in 1..=3 {
            match TcpStream::connect(&self.addr).await {
                Ok(mut stream) => {
                    if let Err(e) = stream.write_all(b"REFRESH").await {
                        tracing::error!("{e}");
                    } else {
                        tracing::info!("Resfresh notification sended");
                        return;
                    }
                }
                Err(e) => {
                    tracing::warn!("Connection attempt {} to watchdog failed: {}", attempt, e);
                    if attempt < 3 {
                        sleep(Duration::from_millis(200)).await;
                    }
                }
            }
        }
        tracing::error!("Failed to notify watchdog after 3 attempts");
    }
}
