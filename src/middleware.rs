use crate::error::ApiError;
use actix_web::dev::ServiceRequest;
use actix_web::web;

static AUTH_TOKEN: &str = "X-auth-token";
