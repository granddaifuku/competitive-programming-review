use super::error::ApiError;
use actix_web::web;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    user_name: String,
    email: String,
    password: String,
}

#[allow(dead_code)]
pub async fn sign_up(pool: web::Data<PgPool>, form: web::Form<NewUser>) -> Result<(), ApiError> {
    match is_already_regiseterd(pool.get_ref(), &form.user_name).await {
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

async fn regist_temporarily(pool: &PgPool, user: NewUser) {}
