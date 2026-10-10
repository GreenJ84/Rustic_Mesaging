use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::Serialize;

/// BOUNDARY LAYER: Broad API Error Edge
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    NotAllowed,
    Conflict(String),
    UnprocessableEntity(String),
    TooManyRequests,
    InternalError,
    NotImplemented,
}

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn status(&self) -> Status {
        match self {
            Self::BadRequest(_) => Status::BadRequest,
            Self::Unauthorized(_) => Status::Unauthorized,
            Self::Forbidden(_) => Status::Forbidden,
            Self::NotFound(_) => Status::NotFound,
            Self::NotAllowed => Status::MethodNotAllowed,
            Self::Conflict(_) => Status::Conflict,
            Self::UnprocessableEntity(_) => Status::UnprocessableEntity,
            Self::TooManyRequests => Status::TooManyRequests,
            Self::InternalError => Status::InternalServerError,
            Self::NotImplemented => Status::NotImplemented,
        }
    }

    fn public_body(self) -> ApiErrorBody {
        let (code, message) = match self {
            Self::BadRequest(msg) => ("bad_request", msg),
            Self::Unauthorized(msg) => ("unauthorized", msg),
            Self::Forbidden(msg) => ("forbidden", msg),
            Self::NotFound(msg) => ("not_found", msg),
            Self::NotAllowed => ("not_allowed", "This action is not allowed".to_string()),
            Self::Conflict(msg) => ("conflict", msg),
            Self::UnprocessableEntity(msg) => ("unprocessable_entity", msg),
            Self::TooManyRequests => ("too_many_requests", "Too many requests".to_string()),
            Self::InternalError => ("internal_error", "An internal error occurred".to_string()),
            Self::NotImplemented => (
                "not_implemented",
                "This feature is not implemented".to_string(),
            ),
        };

        ApiErrorBody { code, message }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        rocket::response::status::Custom(self.status(), Json(self.public_body()))
            .respond_to(request)
    }
}
