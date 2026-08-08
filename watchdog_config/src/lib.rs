use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ConfigErrors {
    #[error("failed to load database config: {0}")]
    DatabaseError(envy::Error),
    #[error("failed to load server config: {0}")]
    ServerError(envy::Error),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

impl Config {
    fn new(db_config: DatabaseConfig, server_config: ServerConfig) -> Self {
        Self {
            database: db_config,
            server: server_config,
        }
    }

    pub fn db_url(&self) -> String {
        self.database.url()
    }
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    user: String,
    host: String,
    password: String,
    port: u16,
    db_name: String,
    pub schema_name: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.db_name
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

pub fn config() -> Result<Config, ConfigErrors> {
    dotenvy::dotenv().ok();

    let db_config: DatabaseConfig = envy::prefixed("DATABASE_")
        .from_env()
        .map_err(ConfigErrors::DatabaseError)?;

    let server_config: ServerConfig = envy::prefixed("SERVER_")
        .from_env()
        .map_err(ConfigErrors::ServerError)?;

    Ok(Config::new(db_config, server_config))
}
