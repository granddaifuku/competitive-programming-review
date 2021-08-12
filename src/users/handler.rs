use super::infrastructures;
use super::model::{LoginUser, NewUser};
use crate::error::{extract_field, ApiError};
use actix_web::{get, post, web, HttpResponse};
use anyhow::Result;
use bcrypt::verify;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

#[post("/sign-up")]
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
    match infrastructures::is_already_registered(pool.get_ref(), &form.user_name).await {
        Ok(f) => {
            if f {
                return Ok(HttpResponse::Accepted().json(""));
            }
        }
        Err(_) => return Err(ApiError::InternalError),
    };

    // check the user is already temporarily registered
    match infrastructures::is_already_registered_temporarily(pool.get_ref(), &form.user_name).await
    {
        Ok(f) => {
            if f {
                return Ok(HttpResponse::Ok().json(""));
            }
        }
        Err(_) => return Err(ApiError::InternalError),
    };

    let uid = Uuid::new_v4();

    // send mail
    match infrastructures::send_mail(&form.user_name, &form.email, &uid).await {
        Ok(f) => {
            if !f {
                return Err(ApiError::BadRequest);
            }
        }
        Err(_) => return Err(ApiError::InternalError),
    }

    // insert the user to temporarily registered users table
    let new_user = form.into_inner();
    match infrastructures::register_temporarily(pool.get_ref(), new_user, uid).await {
        Ok(_) => (),
        Err(_) => return Err(ApiError::InternalError),
    };

    Ok(HttpResponse::Ok().json(""))
}

#[get("/verify/{uid}")]
pub async fn verify_user(
    pool: web::Data<PgPool>,
    web::Path(uid): web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    // extract the user from the temporarily registered table
    match infrastructures::extract_temporarily_table(pool.get_ref(), &uid).await {
        Err(f) => {
            if f {
                return Err(ApiError::BadRequest);
            } else {
                return Err(ApiError::InternalError);
            }
        }
        Ok(user) => {
            // register user
            match infrastructures::register_user(pool.get_ref(), user, &uid).await {
                Ok(_) => (),
                Err(_) => return Err(ApiError::InternalError),
            }
        }
    }
    Ok(HttpResponse::Ok().json(""))
}

#[post("/log-in")]
pub async fn log_in(
    pool: web::Data<PgPool>,
    user_info: web::Json<LoginUser>,
) -> Result<HttpResponse, ApiError> {
    match user_info.validate() {
        Ok(_) => (),
        Err(e) => {
            return Err(ApiError::ValidationError {
                fields: extract_field(e),
            })
        }
    }

    // search the user from users table
    // TODO consider the way to distinguish BadRequest in the following
    match infrastructures::search_user(pool.get_ref(), &user_info.user_name).await {
        Ok(user) => match verify(&user_info.user_name, &user.password) {
            Ok(_) => {
                // Issue the Session ID and store it to Cookie
            }
            Err(_) => {
                return Err(ApiError::BadRequest);
            }
        },
        Err(_) => return Err(ApiError::BadRequest),
    }

    Ok(HttpResponse::Ok().json(""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config, utils};
    use actix_web::{body::Body, test, App};
    use bcrypt::verify;
    use chrono::Utc;
    use serde_json::json;

    #[actix_rt::test]
    async fn user_name_invalid_min_length() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        let user = NewUser {
            user_name: "".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(400, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(
            &Body::from(
                json!({"code": 400, "message": "validation error on field: [\"user_name\"]"})
            ),
            resp_body
        );
    }

    #[actix_rt::test]
    async fn user_name_invalid_max_length() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        let user = NewUser {
    		user_name: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
    		email: "test@gmail.com".to_string(),
    		password: "password".to_string(),
    	};
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(400, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(
            &Body::from(
                json!({"code": 400, "message": "validation error on field: [\"user_name\"]"})
            ),
            resp_body
        );
    }

    #[actix_rt::test]
    async fn user_name_invalid_character() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        let user = NewUser {
            user_name: "aaaあaaa".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(400, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(
            &Body::from(
                json!({"code": 400, "message": "validation error on field: [\"user_name\"]"})
            ),
            resp_body
        );
    }

    #[actix_rt::test]
    async fn email_invalid() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        let user = NewUser {
            user_name: "user_name".to_string(),
            email: "invalid_mail_example".to_string(),
            password: "password".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(400, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(
            &Body::from(json!({"code": 400, "message": "validation error on field: [\"email\"]"})),
            resp_body
        );
    }

    #[actix_rt::test]
    async fn password_invalid_min_length() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        let user = NewUser {
            user_name: "user_name".to_string(),
            email: "test@gmail.com".to_string(),
            password: "".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(400, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(
            &Body::from(
                json!({"code": 400, "message": "validation error on field: [\"password\"]"})
            ),
            resp_body
        );
    }

    #[actix_rt::test]
    async fn password_invalid_max_length() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        let user = NewUser {
    		user_name: "user_name".to_string(),
    		email: "test@gmail.com".to_string(),
    		password: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
    	};
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(400, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(
            &Body::from(
                json!({"code": 400, "message": "validation error on field: [\"password\"]"})
            ),
            resp_body
        );
    }

    #[actix_rt::test]
    async fn password_invalid_character() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        let user = NewUser {
            user_name: "user_name".to_string(),
            email: "test@gmail.com".to_string(),
            password: "aaaあaaa".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(400, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(
            &Body::from(
                json!({"code": 400, "message": "validation error on field: [\"password\"]"})
            ),
            resp_body
        );
    }

    #[actix_rt::test]
    async fn sign_up_already_registered() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        // insert predataset
        let uid = Uuid::new_v4();
        sqlx::query(r#"INSERT INTO users (id, user_name, password, email, uid) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1)"#)
			.bind(&uid)
			.execute(&pool)
			.await
			.unwrap();
        let user = NewUser {
            user_name: "test_user".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(202, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(&Body::from(json!("")), resp_body);

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn sign_up_already_registered_temporarily() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        // insert predataset
        let uid = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(r#"INSERT INTO tmp_users (id, user_name, password, email, uid, created_at) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1, $2)"#)
			.bind(&uid)
			.bind(now)
			.execute(&pool)
			.await
			.unwrap();
        let user = NewUser {
            user_name: "test_user".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(200, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(&Body::from(json!("")), resp_body);

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn sign_up_failed_mail_sending() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        let user = NewUser {
            user_name: "test_user".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(400, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(
            &Body::from(json!({"code": 400, "message": "bad request"})),
            resp_body
        );
    }

    #[ignore]
    #[actix_rt::test]
    async fn sign_up_ok() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(sign_up)).await;
        let user = NewUser {
            user_name: "test_user".to_string(),
            email: "test@gmail.com".to_string(),
            password: "password".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/sign-up")
            .set_form(&user)
            .to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(200, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(&Body::from(json!("")), resp_body);

        // check the user is inserted tmp_users table.
        let tmp_user = sqlx::query!("SELECT * FROM tmp_users")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(1, tmp_user.len());
        assert_eq!("test_user".to_string(), tmp_user[0].user_name);
        assert_eq!("test@gmail.com".to_string(), tmp_user[0].email);
        assert!(verify("password", &tmp_user[0].password).unwrap());

        utils::clear_table(&pool).await.unwrap();
    }

    #[actix_rt::test]
    async fn verify_user_not_exist() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();
        let mut app = test::init_service(App::new().data(pool.clone()).service(verify_user)).await;
        let uid = Uuid::new_v4();
        let uri = format!("/verify/{}", uid);
        let req = test::TestRequest::get().uri(&uri).to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(400, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(
            &Body::from(json!({"code": 400, "message": "bad request"})),
            resp_body
        );
    }

    #[actix_rt::test]
    async fn verify_user_ok() {
        let config = config::Config::new();
        let pool = PgPool::connect(&config.database_url).await.unwrap();

        // insert predataset
        let uid = Uuid::new_v4();
        let now = chrono::Utc::now();
        sqlx::query(r#"INSERT INTO tmp_users (id, user_name, password, email, uid, created_at) VALUES (0, 'test_user', 'password', 'test@gmail.com', $1, $2)"#)
			.bind(&uid)
			.bind(now)
			.execute(&pool)
			.await
			.unwrap();

        let mut app = test::init_service(App::new().data(pool.clone()).service(verify_user)).await;
        let uri = format!("/verify/{}", &uid);
        let req = test::TestRequest::get().uri(&uri).to_request();
        let mut resp = test::call_service(&mut app, req).await;
        assert_eq!(200, resp.status());
        let resp_body = resp.take_body();
        let resp_body = resp_body.as_ref().unwrap();
        assert_eq!(&Body::from(json!("")), resp_body);

        // check the user is deleted from tmp_users table.
        let tmp_user = sqlx::query!("SELECT * FROM tmp_users")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(0, tmp_user.len());

        // check the user is inserted to the users table.
        let user = sqlx::query!("SELECT * FROM users")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(1, user.len());
        assert_eq!("test_user".to_string(), user[0].user_name);
        assert_eq!("test@gmail.com".to_string(), user[0].email);
        assert_eq!("password".to_string(), user[0].password);
        assert_eq!(uid, user[0].uid);

        utils::clear_table(&pool).await.unwrap();
    }
}
