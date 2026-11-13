use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use serde::Serialize;
use std::{error::Error, fmt::Display};

/// API boundary error represented as an HTTP status and JSON error envelope.
#[derive(Debug)]
pub enum ApiError {
    /// The request was malformed or failed basic validation.
    BadRequest(String),
    /// Authentication is required or credentials are invalid.
    Unauthorized(String),
    /// The caller is authenticated but lacks permission.
    Forbidden(String),
    /// The requested resource does not exist.
    NotFound(String),
    /// The HTTP method is not supported for the requested resource.
    NotAllowed,
    /// The request exceeded the server's processing time limit.
    RequestTimeout,
    /// The request conflicts with current application state.
    Conflict(String),
    /// The request was valid but failed domain validation.
    UnprocessableEntity(String),
    /// The caller exceeded an applicable rate limit.
    TooManyRequests,
    /// A required upstream or dependent operation failed.
    FailedDependency,
    /// An unexpected internal failure occurred.
    InternalError,
    /// The requested capability is not implemented.
    NotImplemented,
}
impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "{}: {}", self.status(), self.message()),
            Self::Unauthorized(msg) => write!(f, "{}: {}", self.status(), self.message()),
            Self::Forbidden(msg) => write!(f, "{}: {}", self.status(), self.message()),
            Self::NotFound(msg) => write!(f, "{}: {}", self.status(), self.message()),
            Self::NotAllowed => write!(f, "{}: {}", self.status(), self.message()),
            Self::RequestTimeout => write!(f, "{}: {}", self.status(), self.message()),
            Self::Conflict(msg) => write!(f, "{}: {}", self.status(), self.message()),
            Self::UnprocessableEntity(msg) => write!(f, "{}: {}", self.status(), self.message()),
            Self::TooManyRequests => write!(f, "{}: {}", self.status(), self.message()),
            Self::FailedDependency => write!(f, "{}: {}", self.status(), self.message()),
            Self::InternalError => write!(f, "{}: {}", self.status(), self.message()),
            Self::NotImplemented => write!(f, "{}: {}", self.status(), self.message()),
        }
    }
}
impl Error for ApiError {}

impl ApiError {
    /// Returns the HTTP status associated with this error.
    pub fn status(&self) -> Status {
        match self {
            Self::BadRequest(_) => Status::BadRequest,
            Self::Unauthorized(_) => Status::Unauthorized,
            Self::Forbidden(_) => Status::Forbidden,
            Self::NotFound(_) => Status::NotFound,
            Self::NotAllowed => Status::MethodNotAllowed,
            Self::RequestTimeout => Status::RequestTimeout,
            Self::Conflict(_) => Status::Conflict,
            Self::UnprocessableEntity(_) => Status::UnprocessableEntity,
            Self::TooManyRequests => Status::TooManyRequests,
            Self::FailedDependency => Status::FailedDependency,
            Self::InternalError => Status::InternalServerError,
            Self::NotImplemented => Status::NotImplemented,
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> String {
        match self {
            Self::BadRequest(msg) => format!("Bad request: {msg}"),
            Self::Unauthorized(msg) => format!("Unauthorized: {msg}"),
            Self::Forbidden(msg) => format!("Forbidden: {msg}"),
            Self::NotFound(msg) => format!("Not found: {msg}"),
            Self::NotAllowed => String::from("Method not allowed"),
            Self::RequestTimeout => String::from("Request timeout"),
            Self::Conflict(msg) => format!("Conflict: {msg}"),
            Self::UnprocessableEntity(msg) => format!("Unprocessable entity: {msg}"),
            Self::TooManyRequests => String::from("Too many requests"),
            Self::FailedDependency => String::from("Failed dependency"),
            Self::InternalError => String::from("Internal server error"),
            Self::NotImplemented => String::from("Not implemented"),
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        rocket::response::status::Custom(self.status(), Json(self.message())).respond_to(request)
    }
}

#[cfg(test)]
mod tests {
    use super::ApiError;
    use rocket::http::Status;

    #[test]
    fn maps_error_variants_to_expected_statuses() {
        let cases = [
            (ApiError::BadRequest(String::new()), Status::BadRequest),
            (ApiError::Unauthorized(String::new()), Status::Unauthorized),
            (ApiError::Forbidden(String::new()), Status::Forbidden),
            (ApiError::NotFound(String::new()), Status::NotFound),
            (ApiError::NotAllowed, Status::MethodNotAllowed),
            (ApiError::RequestTimeout, Status::RequestTimeout),
            (ApiError::Conflict(String::new()), Status::Conflict),
            (
                ApiError::UnprocessableEntity(String::new()),
                Status::UnprocessableEntity,
            ),
            (ApiError::TooManyRequests, Status::TooManyRequests),
            (ApiError::FailedDependency, Status::FailedDependency),
            (ApiError::InternalError, Status::InternalServerError),
            (ApiError::NotImplemented, Status::NotImplemented),
        ];

        for (error, expected) in cases {
            assert_eq!(error.status(), expected);
        }
    }

    #[test]
    fn preserves_public_messages_and_hides_internal_messages() {
        let error = ApiError::BadRequest("invalid username".to_string());
        assert_eq!(error.status().code, 400);
        assert_eq!(error.message(), "Bad request: invalid username");

        let error = ApiError::InternalError;
        assert_eq!(error.status().code, 500);
        assert_eq!(error.message(), "Internal server error");
    }

    #[test]
    fn uses_stable_codes_for_non_message_errors() {
        assert_eq!(ApiError::NotAllowed.status().code, 405);
        assert_eq!(ApiError::FailedDependency.status().code, 424);
        assert_eq!(ApiError::NotImplemented.status().code, 501);
    }
}
