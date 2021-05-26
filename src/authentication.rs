use super::error::{extract_field, ApiError};
use actix_web::web;
use anyhow::Result;
use chrono::Utc;
use lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

lazy_static! {
    static ref RE_ALP_NUM_SYM: Regex = Regex::new(r"^[a-zA-Z0-9!-/:-@¥\[-`{-~]*$").unwrap();
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewUser {
    #[validate(length(min = 1, max = 100), regex(path = "RE_ALP_NUM_SYM"))]
    user_name: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 1, max = 100), regex(path = "RE_ALP_NUM_SYM"))]
    password: String,
}

#[allow(dead_code)]
pub async fn sign_up(pool: web::Data<PgPool>, form: web::Form<NewUser>) -> Result<(), ApiError> {
    match form.validate() {
        Ok(_) => (),
        Err(e) => {
            return Err(ApiError::ValidationError {
                fields: extract_field(e),
            })
        }
    }
    match is_already_regiseterd(pool.get_ref(), &form.user_name).await {
        Ok(f) => {
            if f {
                return Ok(());
            }
        }
        Err(_) => return Err(ApiError::InternalError),
    };

    match is_already_registered_temporarily(pool.get_ref(), &form.user_name).await {
        Ok(f) => {
            if f {
                return Ok(());
            }
        }
        Err(_) => return Err(ApiError::InternalError),
    };

    Ok(())
}

async fn is_already_regiseterd(pool: &PgPool, user_name: &str) -> Result<bool> {
    let user = sqlx::query("SELECT * FROM users WHERE user_name = ?")
        .bind(user_name)
        .fetch_optional(pool)
        .await?;
    match user {
        None => Ok(false),
        _ => Ok(true),
    }
}

async fn is_already_registered_temporarily(pool: &PgPool, user_name: &str) -> Result<bool> {
    let user = sqlx::query("SELECT * FROM tmp_users WHERE user_name = ?")
        .bind(user_name)
        .fetch_optional(pool)
        .await?;
    match user {
        None => Ok(false),
        _ => Ok(true),
    }
}

async fn register_temporarily(pool: &PgPool, user: NewUser) -> Result<()> {
    let uid = Uuid::new_v4();
    let now = Utc::now().timestamp();
    sqlx::query(r#"INSERT INTO tmp_user (user_name, password, uid, email, created_at) VALUES (?, ?, ?, ?, ?)"#)
		.bind(user.user_name)
		.bind(user.password)
		.bind(user.email)
		.bind(uid)
		.bind(now)
		.execute(pool)
		.await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    #[actix_rt::test]
    async fn user_name_invalid_min_length() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await;
        let user = NewUser {
            user_name: "".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let form = web::Form(user);
        let p = web::Data::new(pool.unwrap());
        let expected = Err(ApiError::ValidationError {
            fields: vec!["user_name".to_string()],
        });
        let actual = sign_up(p, form).await;
        assert_eq!(expected, actual);
    }

    #[actix_rt::test]
    async fn user_name_invalid_max_length() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await;
        let user = NewUser {
            user_name: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let form = web::Form(user);
        let p = web::Data::new(pool.unwrap());
        let expected = Err(ApiError::ValidationError {
            fields: vec!["user_name".to_string()],
        });
        let actual = sign_up(p, form).await;
        assert_eq!(expected, actual);
    }
    #[actix_rt::test]
    async fn user_name_invalid_character() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await;
        let user = NewUser {
            user_name: "aaaあaaa".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let form = web::Form(user);
        let p = web::Data::new(pool.unwrap());
        let expected = Err(ApiError::ValidationError {
            fields: vec!["user_name".to_string()],
        });
        let actual = sign_up(p, form).await;
        assert_eq!(expected, actual);
    }

    #[actix_rt::test]
    async fn email_invalid() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await;
        let user = NewUser {
            user_name: "user_name".to_string(),
            email: "invalid_mail_example".to_string(),
            password: "password".to_string(),
        };
        let form = web::Form(user);
        let p = web::Data::new(pool.unwrap());
        let expected = Err(ApiError::ValidationError {
            fields: vec!["email".to_string()],
        });
        let actual = sign_up(p, form).await;
        assert_eq!(expected, actual);
    }

    #[actix_rt::test]
    async fn password_invalid_min_length() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await;
        let user = NewUser {
            user_name: "user_name".to_string(),
            email: "test@gmail.com".to_string(),
            password: "".to_string(),
        };
        let form = web::Form(user);
        let p = web::Data::new(pool.unwrap());
        let expected = Err(ApiError::ValidationError {
            fields: vec!["password".to_string()],
        });
        let actual = sign_up(p, form).await;
        assert_eq!(expected, actual);
    }

    #[actix_rt::test]
    async fn password_invalid_max_length() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await;
        let user = NewUser {
            user_name: "user_name".to_string(),
            email: "test@gmail.com".to_string(),
            password: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        };
        let form = web::Form(user);
        let p = web::Data::new(pool.unwrap());
        let expected = Err(ApiError::ValidationError {
            fields: vec!["password".to_string()],
        });
        let actual = sign_up(p, form).await;
        assert_eq!(expected, actual);
    }

    #[actix_rt::test]
    async fn password_invalid_character() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await;
        let user = NewUser {
            user_name: "user_name".to_string(),
            email: "test@gmail.com".to_string(),
            password: "aaaあaaa".to_string(),
        };
        let form = web::Form(user);
        let p = web::Data::new(pool.unwrap());
        let expected = Err(ApiError::ValidationError {
            fields: vec!["password".to_string()],
        });
        let actual = sign_up(p, form).await;
        assert_eq!(expected, actual);
    }
}
