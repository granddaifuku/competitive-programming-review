mod config;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;

#[rocket::main]
async fn main() -> Result<()> {
    let config = config::Config::new();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    rocket::build().manage(pool).launch().await?;

    Ok(())
}
