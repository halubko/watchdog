use std::process;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    server: ServerConfig,
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
    max_connections: u32,
}

impl DatabaseConfig {
    fn url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.db_name
        )
    }
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

pub fn config() -> Config {
    dotenvy::dotenv().ok();

    let db_config: DatabaseConfig = match envy::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{:#?}", error);
            process::exit(1)
        }
    };

    let server_config: ServerConfig = match envy::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{:#?}", error);
            process::exit(1)
        }
    };

    Config::new(db_config, server_config)
}
