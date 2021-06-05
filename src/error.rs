use actix_web::{error, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use serde::Serialize;
use validator::ValidationErrors;

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

// TODO remove the attribute
#[allow(dead_code)]
#[derive(Debug, Display, Error, PartialEq)]
pub enum ApiError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadRequest,

    #[display(fmt = "timeout")]
    Timeout,

    #[display(fmt = "validation error on field: {:?}", fields)]
    ValidationError { fields: Vec<String> },
}

impl error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
        };
        HttpResponse::build(self.status_code()).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest => StatusCode::BAD_REQUEST,
            ApiError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            ApiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
        }
    }
}

pub fn extract_field(err: ValidationErrors) -> Vec<String> {
    let mut fields: Vec<String> = Vec::new();
    for key in err.into_errors().keys() {
        fields.push(key.to_string());
    }

    fields
}
