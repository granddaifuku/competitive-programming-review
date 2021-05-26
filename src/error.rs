use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use validator::ValidationErrors;

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
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
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
