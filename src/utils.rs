use anyhow::Result;
use lazy_static;
use regex::Regex;
use sqlx::PgPool;

lazy_static! {
    // alphabet, number, symbol
    pub static ref RE_ALP_NUM_SYM: Regex = Regex::new(r"^[a-zA-Z0-9!-/:-@¥\[-`{-~]*$").unwrap();
}

// Clear table for testing
pub async fn clear_table(pool: &PgPool) -> Result<()> {
    let tables: Vec<String> = vec![
        "users".to_string(),
        "reviews".to_string(),
        "tmp_users".to_string(),
    ];
    for table in tables {
        // cannot bind table and thus prepare sql string
        let sql = format!("DELETE FROM {}", table);
        sqlx::query(&sql).bind(table).execute(pool).await.unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use chrono::Utc;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn clear_table_test() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let uid = Uuid::new_v4();
        let now = Utc::now();

        // insert pre-dataset
        sqlx::query(r#"INSERT INTO users (id, user_name, password, email, uid) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1)"#)
			.bind(uid)
			.execute(&pool)
			.await
			.unwrap();

        sqlx::query(r#"INSERT INTO reviews (id, problem_name, url, memo, uid, platform, created_at, updated_at) VALUES (0, 'test_prob_name', 'test_url', 'test_memo', $1, 'test_platform', $2, $3)"#)
			.bind(uid)
			.bind(now)
			.bind(now)
			.execute(&pool)
			.await
			.unwrap();

        sqlx::query(r#"INSERT INTO tmp_users (id, user_name, password, email, uid, created_at) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1, $2)"#)
			.bind(uid)
			.bind(now)
			.execute(&pool)
			.await
			.unwrap();

        // check correctly inserted
        let users_before = sqlx::query("SELECT * FROM users")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(1, users_before.len());
        let reviews_before = sqlx::query("SELECT * FROM reviews")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(1, reviews_before.len());
        let tmp_users_before = sqlx::query("SELECT * FROM tmp_users")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(1, tmp_users_before.len());

        clear_table(&pool).await.unwrap();

        // check correctly deleted
        let users_after = sqlx::query("SELECT * FROM users")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(0, users_after.len());
        let reviews_after = sqlx::query("SELECT * FROM reviews")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(0, reviews_after.len());
        let tmp_users_after = sqlx::query("SELECT * FROM tmp_users")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(0, tmp_users_after.len());
    }
}
