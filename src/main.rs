mod config;
mod db;

use sqlx::postgres::PgPoolOptions;

fn main() {
    let config = config::Config::new();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url);

    rocket::ignite().launch();
}
