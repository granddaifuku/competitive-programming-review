use std::env;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn new() -> Config {
        let database_url = env::var("DATABASE_URL").unwrap();
        Config { database_url }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let expected = Config {
            database_url: "postgres://postgres:password@localhost:5432/test".to_string(),
        };
        let actual = Config::new();
        assert_eq!(expected, actual);
    }
}
