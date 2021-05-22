#[macro_use]
extern crate rocket;
mod authentication;
mod config;

use anyhow::Result;
use sqlx::postgres::PgPool;
use std::sync::Arc;

#[rocket::main]
async fn main() -> Result<()> {
    let config = config::Config::new();
    let pool = {
        let p = PgPool::connect(&config.database_url).await?;
        Arc::new(p)
    };

    rocket::build()
        .manage(pool)
        .mount("/", routes![authentication::sign_up])
        .launch()
        .await?;

    Ok(())
}
