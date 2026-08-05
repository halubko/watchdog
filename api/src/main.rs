#[tokio::main]
async fn main() {
    let config = config::config();

    let mut _connection = db::connect(&config.database).await;
}
