use super::error::{extract_field, ApiError};
use super::utils::RE_ALP_NUM_SYM;
use actix_web::{web, HttpResponse};
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

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
pub async fn sign_up(
    pool: web::Data<PgPool>,
    form: web::Form<NewUser>,
) -> Result<HttpResponse, ApiError> {
    match form.validate() {
        Ok(_) => (),
        Err(e) => {
            return Err(ApiError::ValidationError {
                fields: extract_field(e),
            })
        }
    }

    // check the user is already registered
    match is_already_registered(pool.get_ref(), &form.user_name).await {
        Ok(f) => {
            if f {
                return Ok(HttpResponse::Ok().finish());
            }
        }
        Err(_) => return Err(ApiError::InternalError),
    };

    // check the user is already temporarily registered
    match is_already_registered_temporarily(pool.get_ref(), &form.user_name).await {
        Ok(f) => {
            if f {
                return Ok(HttpResponse::Ok().finish());
            }
        }
        Err(_) => return Err(ApiError::InternalError),
    };

    let uid = Uuid::new_v4();

    // insert the user to temporarily registered users table
    let new_user = form.into_inner();
    match register_temporarily(pool.get_ref(), new_user, uid).await {
        Ok(_) => (),
        Err(_) => return Err(ApiError::InternalError),
    };

    Ok(HttpResponse::Ok().finish())
}

async fn is_already_registered(pool: &PgPool, user_name: &str) -> Result<bool> {
    let user = sqlx::query("SELECT * FROM users WHERE user_name = $1")
        .bind(user_name)
        .fetch_optional(pool)
        .await?;
    match user {
        None => Ok(false),
        _ => Ok(true),
    }
}

async fn is_already_registered_temporarily(pool: &PgPool, user_name: &str) -> Result<bool> {
    let user = sqlx::query("SELECT * FROM tmp_users WHERE user_name = $1")
        .bind(user_name)
        .fetch_optional(pool)
        .await?;
    match user {
        None => Ok(false),
        _ => Ok(true),
    }
}

async fn register_temporarily(pool: &PgPool, user: NewUser, uid: Uuid) -> Result<()> {
    let now = Utc::now();
    let hashed_password = hash(user.password, DEFAULT_COST).unwrap();
    sqlx::query(r#"INSERT INTO tmp_users (user_name, password, uid, email, created_at) VALUES ($1, $2, $3, $4, $5)"#)
		.bind(user.user_name)
		.bind(hashed_password)
		.bind(uid)
		.bind(user.email)
		.bind(now)
		.execute(pool)
		.await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::utils;

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

    #[actix_rt::test]
    async fn is_already_registered_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let uuid_example = Uuid::new_v4();
        sqlx::query(r#"INSERT INTO users (id, user_name, password, email, uid) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1)"#)
    		.bind(uuid_example)
    		.execute(&pool).await.unwrap();
        let expected = true;
        let actual = is_already_registered(&pool, "test_user").await.unwrap();
        assert_eq!(expected, actual);

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn is_already_registered_not_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let uuid_example = Uuid::new_v4();
        sqlx::query(r#"INSERT INTO users (id, user_name, password, email, uid) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1)"#)
    		.bind(uuid_example)
    		.execute(&pool).await.unwrap();
        let expected = false;
        let actual = is_already_registered(&pool, "test_user_not_exist")
            .await
            .unwrap();
        assert_eq!(expected, actual);

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn is_already_registered_temporarily_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let uuid_example = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(r#"INSERT INTO tmp_users (id, user_name, password, email, uid, created_at) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1, $2)"#)
    		.bind(uuid_example)
			.bind(now)
    		.execute(&pool).await.unwrap();
        let expected = true;
        let actual = is_already_registered_temporarily(&pool, "test_user")
            .await
            .unwrap();
        assert_eq!(expected, actual);

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn is_already_registered_temporarily_not_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let uuid_example = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(r#"INSERT INTO tmp_users (id, user_name, password, email, uid, created_at) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1, $2)"#)
    		.bind(uuid_example)
			.bind(now)
    		.execute(&pool).await.unwrap();
        let expected = false;
        let actual = is_already_registered_temporarily(&pool, "test_user_not_exist")
            .await
            .unwrap();
        assert_eq!(expected, actual);

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn register_temporarily_create() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let user = NewUser {
            user_name: "user_name".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let uid = Uuid::new_v4();
        let uid_clone = uid.clone();
        let tmp_users_before = sqlx::query("SELECT * FROM tmp_users")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(0, tmp_users_before.len());
        register_temporarily(&pool, user, uid).await.unwrap();

        let tmp_users_after = sqlx::query!("SELECT * FROM tmp_users where user_name = 'user_name'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!("user_name".to_string(), tmp_users_after.user_name);
        assert_eq!("test@gmail.com".to_string(), tmp_users_after.email);
        assert_eq!(true, verify("password", &tmp_users_after.password).unwrap());
        assert_eq!(uid_clone, tmp_users_after.uid);

        utils::clear_table(&pool).await.unwrap();
    }
}
