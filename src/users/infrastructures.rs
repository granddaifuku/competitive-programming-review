use super::model::NewUser;
use crate::config;
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn is_already_registered(pool: &PgPool, user_name: &str) -> Result<bool> {
    let user = sqlx::query("SELECT * FROM users WHERE user_name = $1")
        .bind(user_name)
        .fetch_optional(pool)
        .await?;
    match user {
        None => Ok(false),
        _ => Ok(true),
    }
}

pub async fn is_already_registered_temporarily(pool: &PgPool, user_name: &str) -> Result<bool> {
    let user = sqlx::query("SELECT * FROM tmp_users WHERE user_name = $1")
        .bind(user_name)
        .fetch_optional(pool)
        .await?;
    match user {
        None => Ok(false),
        _ => Ok(true),
    }
}

pub async fn send_mail(user_name: &str, mail_address: &str, uid: &Uuid) -> Result<bool> {
    let config = config::Config::new();
    let body = format!(
        "Hi {}! Verify your account by clicking on https://verify/uid={}",
        user_name, uid
    );
    let email = Message::builder()
        .from(
            "Competitive Programming Review Admin <info@granddaifuku.com>"
                .parse()
                .unwrap(),
        )
        .to(mail_address.parse().unwrap())
        .subject("[DO NOT REPLY] SIGN-UP")
        .body(String::from(body))
        .unwrap();
    let creds = Credentials::new(config.smtp_username, config.smtp_password);

    let mailer = SmtpTransport::starttls_relay(&config.mailer)
        .unwrap()
        .credentials(creds)
        .build();
    match mailer.send(&email) {
        Ok(_) => Ok(true),
        _ => Ok(false),
    }
}

pub async fn register_temporarily(pool: &PgPool, user: NewUser, uid: Uuid) -> Result<()> {
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
    use crate::utils;

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
    #[ignore]
    async fn send_mail_ok() {
        let user_name = "dummy_user";
        let mail_address = "dummy_mail";
        let uid = Uuid::new_v4();
        let expected = true;
        let actual = send_mail(user_name, mail_address, &uid).await.unwrap();
        assert_eq!(expected, actual);
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
