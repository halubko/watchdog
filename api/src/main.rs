#[tokio::main]
async fn main() {
    let config = config::config();
    let url = &config.db_url();

    let mut _connection = db::connect(url, &config.database.schema_name).await;
}
