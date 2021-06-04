use super::infrastructures;
use super::model::NewUser;
use crate::error::{extract_field, ApiError};
use actix_web::{get, post, web, HttpResponse};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

#[allow(dead_code)]
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
                return Ok(HttpResponse::Ok().json(""));
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

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use actix_web::{body::Body, test, App};
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
}
