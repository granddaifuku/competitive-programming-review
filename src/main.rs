mod authentication;
mod config;
mod error;

#[macro_use]
extern crate lazy_static;

use actix_web::{App, HttpServer};
use anyhow::Result;
use sqlx::postgres::PgPool;

#[actix_web::main]
#[allow(unused_must_use)]
async fn main() -> Result<()> {
    let config = config::Config::new();
    let pool = PgPool::connect(&config.database_url).await?;
    HttpServer::new(move || App::new().data(pool.clone()))
        .bind("127.0.0.1:8000")?
        .run()
        .await;

    Ok(())
}
