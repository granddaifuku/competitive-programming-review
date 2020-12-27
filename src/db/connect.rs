use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::ConnectionError;
use dotenv::dotenv;
use std::env;

fn establish_connection(db_url: String) -> Result<PgConnection, ConnectionError> {
    PgConnection::establish(&db_url)
}

fn database_url() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[should_panic]
    fn test_establish_connection_fail() {
        establish_connection("InvalidDatabaseURL".to_string()).unwrap();
    }
}
