mod config;

use actix_web::{App, HttpServer};
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
#[allow(unused_variables)]
#[allow(unused_must_use)]
async fn main() -> Result<()> {
    let config = config::Config::new();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    HttpServer::new(App::new)
        .bind("127.0.0.1:8000")?
        .run()
        .await;

    Ok(())
}
