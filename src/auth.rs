use crate::error::ApiError;
use actix_web::dev::ServiceRequest;
use actix_web::web;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

async fn validator(
    req: ServiceRequest,
    pool: web::Data<PgPool>,
    auth: BearerAuth,
) -> Result<ServiceRequest, ApiError> {
    let token = auth.token();
    let auth_info = sqlx::query!("SELECT * FROM auth WHERE token = $1", token)
        .fetch_optional(pool.get_ref())
        .await;
    match auth_info {
        Err(_) => Err(ApiError::InternalError),
        Ok(c) => match c {
            None => Err(ApiError::Unauthorized),
            Some(c) => {
                // Assume the session ID lives in 30 days
                let timestamp: DateTime<Utc> = c.created_at;
                let now = Utc::now();
                let duration = now - timestamp;
                if duration.num_days() > 7 {
                    return Err(ApiError::Unauthorized);
                }

                Ok(req)
            }
        },
    }
}
