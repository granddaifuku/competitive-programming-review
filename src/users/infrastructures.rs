use super::model::NewUser;
use crate::config;
use anyhow::Result;
use bcrypt::{hash, DEFAULT_COST};
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
        "Hi {}! Verify your account by clicking on https://verify/{}",
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
        .body(body)
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

pub async fn extract_temporarily_table(pool: &PgPool, uid: &Uuid) -> Result<NewUser, bool> {
    let user = sqlx::query!(
        "SELECT user_name, password, email FROM tmp_users WHERE uid = $1",
        uid
    )
    .fetch_optional(pool)
    .await;
    if user.is_err() {
        return Err(false);
    }

    match user.unwrap() {
        None => Err(true),
        Some(u) => {
            if sqlx::query("DELETE FROM tmp_users WHERE uid = $1")
                .bind(uid)
                .execute(pool)
                .await
                .is_err()
            {
                return Err(false);
            }
            let new_user = NewUser {
                user_name: u.user_name,
                email: u.email,
                password: u.password,
            };
            Ok(new_user)
        }
    }
}

pub async fn register_user(pool: &PgPool, user: NewUser, uid: &Uuid) -> Result<()> {
    sqlx::query(r#"INSERT INTO users (user_name, password, email, uid) VALUES ($1, $2, $3, $4)"#)
        .bind(user.user_name)
        .bind(user.password)
        .bind(user.email)
        .bind(uid)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn search_user(pool: &PgPool, user_name: &str) -> Result<NewUser, ()> {
    let user = sqlx::query!(
        "SELECT user_name, password, email FROM users WHERE user_name = $1",
        user_name
    )
    .fetch_optional(pool)
    .await;
    match user.unwrap() {
        None => Err(()),
        Some(u) => {
            let user = NewUser {
                user_name: u.user_name,
                email: u.email,
                password: u.password,
            };
            Ok(user)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;
    use bcrypt::verify;

    #[actix_rt::test]
    async fn is_already_registered_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();

        // insert predataset
        let uid_example = Uuid::new_v4();
        sqlx::query(r#"INSERT INTO users (id, user_name, password, email, uid) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1)"#)
    		.bind(uid_example)
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

        // insert predataset
        let uid_example = Uuid::new_v4();
        sqlx::query(r#"INSERT INTO users (id, user_name, password, email, uid) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1)"#)
    		.bind(uid_example)
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

        // insert predataset
        let uid_example = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(r#"INSERT INTO tmp_users (id, user_name, password, email, uid, created_at) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1, $2)"#)
    		.bind(uid_example)
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

        // insert predataset
        let uid_example = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(r#"INSERT INTO tmp_users (id, user_name, password, email, uid, created_at) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1, $2)"#)
    		.bind(uid_example)
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
        assert!(verify("password", &tmp_users_after.password).unwrap());
        assert_eq!(uid_clone, tmp_users_after.uid);

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn extract_temporarily_table_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let uid = Uuid::new_v4();
        let now = Utc::now();

        // insert predataset
        sqlx::query(r#"INSERT INTO tmp_users (id, user_name, password, email, uid, created_at) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1, $2)"#)
			.bind(&uid)
			.bind(now)
			.execute(&pool)
			.await
			.unwrap();

        let expected = NewUser {
            user_name: "test_user".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };

        let actual = extract_temporarily_table(&pool, &uid).await.unwrap();
        assert_eq!(expected, actual);
        let user = sqlx::query("SELECT * FROM tmp_users")
            .fetch_optional(&pool)
            .await
            .unwrap();
        // check the user is deleted from the tmp_user table.
        assert!(user.is_none());

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn extract_temporarily_table_not_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let uid = Uuid::new_v4();
        let now = Utc::now();

        // insert predataset
        sqlx::query(r#"INSERT INTO tmp_users (id, user_name, password, email, uid, created_at) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1, $2)"#)
			.bind(&uid)
			.bind(now)
			.execute(&pool)
			.await
			.unwrap();

        let expected = true;
        let uid_not_exist = Uuid::new_v4();
        let actual = extract_temporarily_table(&pool, &uid_not_exist)
            .await
            .unwrap_err();

        assert_eq!(expected, actual);

        let user = sqlx::query("SELECT * FROM tmp_users")
            .fetch_optional(&pool)
            .await
            .unwrap();
        // check the user is still in the table
        assert!(user.is_some());

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn register_user_test() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let uid_example = Uuid::new_v4();

        // check there's no record
        let user_before = sqlx::query("SELECT * FROM users")
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(user_before.is_none());
        let new_user = NewUser {
            user_name: "user_name".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        register_user(&pool, new_user, &uid_example).await.unwrap();

        // check the user is inserted
        let user_after = sqlx::query!("SELECT * FROM users")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(1, user_after.len());
        assert_eq!("user_name".to_string(), user_after[0].user_name);
        assert_eq!("test@gmail.com".to_string(), user_after[0].email);
        assert_eq!("password".to_string(), user_after[0].password);

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn search_user_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();

        let uid = Uuid::new_v4();

        //insert predataset
        sqlx::query(r#"INSERT INTO users (id, user_name, password, email, uid) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1)"#)
			.bind(uid)
			.execute(&pool)
			.await
			.unwrap();
        let expected = NewUser {
            user_name: "test_user".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let actual = search_user(&pool, "test_user").await.unwrap();
        assert_eq!(expected, actual);

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn search_user_not_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let result = search_user(&pool, "test_user").await.is_err();
        assert!(result);
    }
}
