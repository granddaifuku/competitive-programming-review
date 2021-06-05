use std::env;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub database_url: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub mailer: String,
}

impl Config {
    pub fn new() -> Config {
        let database_url = env::var("DATABASE_URL").unwrap();
        let smtp_username = env::var("SMTP_USERNAME").unwrap();
        let smtp_password = env::var("SMTP_PASSWORD").unwrap();
        let mailer = env::var("MAILER").unwrap();
        Config {
            database_url,
            smtp_username,
            smtp_password,
            mailer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let expected = Config {
            database_url: "postgres://postgres:password@localhost:5432/test".to_string(),
            smtp_username: "dummy_username".to_string(),
            smtp_password: "dummy_password".to_string(),
            mailer: "dummy_mailer".to_string(),
        };
        let actual = Config::new();
        assert_eq!(expected, actual);
    }
}
