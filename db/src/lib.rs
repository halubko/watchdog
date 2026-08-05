use std::process;

use sqlx::{Connection, PgConnection, migrate::Migrate};

pub async fn connect(url: &String, schema_name: &str) -> PgConnection {
    let connection = PgConnection::connect(url).await;

    let mut connection = match connection {
        Ok(connection) => {
            println!("DB connected succesfully");
            connection
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1)
        }
    };

    if let Err(e) = connection.create_schema_if_not_exists(schema_name).await {
        eprintln!("Error creating schema {schema_name}: {e}")
    }

    connection
}
