//! Rocket catchers that translate framework failures into API errors.

use rocket::Request;

use super::error::ApiError;

#[catch(400)]
/// Returns the standard JSON response for bad requests.
pub fn bad_request(_req: &Request) -> ApiError {
    ApiError::BadRequest("The request itself was bad".to_string())
}

#[catch(401)]
/// Returns the standard JSON response for unauthenticated requests.
pub fn unauthorized(_req: &Request) -> ApiError {
    ApiError::Unauthorized("Authentication is required".to_string())
}

#[catch(403)]
/// Returns the standard JSON response for forbidden requests.
pub fn forbidden(_req: &Request) -> ApiError {
    ApiError::Forbidden("You are not allowed to perform this action".to_string())
}

#[catch(404)]
/// Returns the standard JSON response for missing routes or resources.
pub fn not_found(req: &Request) -> ApiError {
    ApiError::NotFound(format!("No route or resource was found at '{}'", req.uri()))
}

#[catch(405)]
/// Returns the standard JSON response for unsupported HTTP methods.
pub fn method_not_allowed(_req: &Request) -> ApiError {
    ApiError::NotAllowed
}

#[catch(408)]
/// Returns the standard JSON response for timed-out requests.
pub fn request_timeout(_req: &Request) -> ApiError {
    ApiError::RequestTimeout
}

#[catch(409)]
/// Returns the standard JSON response for request conflicts.
pub fn conflict(req: &Request) -> ApiError {
    ApiError::Conflict(format!(
        "The {} to '{}' conflicts with current state",
        req.method(),
        req.uri()
    ))
}

#[catch(422)]
/// Returns the standard JSON response for unprocessable content.
pub fn unprocessable_content(_req: &Request) -> ApiError {
    ApiError::UnprocessableEntity("".to_string())
}

#[catch(424)]
/// Returns the standard JSON response for failed dependencies.
pub fn failed_dependency(_req: &Request) -> ApiError {
    ApiError::FailedDependency
}

#[catch(429)]
/// Returns the standard JSON response for too many requests.
pub fn too_many_requests(_req: &Request) -> ApiError {
    ApiError::TooManyRequests
}

#[catch(500)]
/// Returns the standard JSON response for unexpected server failures.
pub fn internal_error() -> ApiError {
    ApiError::InternalError
}

#[catch(501)]
/// Returns the standard JSON response for unimplemented features.
pub fn not_implemented() -> ApiError {
    ApiError::NotImplemented
}

#[cfg(test)]
mod tests {
    use super::*;

    use rocket::local::blocking::Client;
    use std::assert_matches;

    #[test]
    fn catchers_map_to_api_errors() {
        // 2. Construct a local client using your rocket instance
        let client = Client::tracked(rocket::build()).expect("valid rocket instance");
        let req = client.req(rocket::http::Method::Get, "/nonexistent");

        let items: [(ApiError, ApiError, u16); 12] = [
            (bad_request(&req), ApiError::BadRequest(String::new()), 400),
            (
                unauthorized(&req),
                ApiError::Unauthorized(String::new()),
                401,
            ),
            (forbidden(&req), ApiError::Forbidden(String::new()), 403),
            (not_found(&req), ApiError::NotFound(String::new()), 404),
            (method_not_allowed(&req), ApiError::NotAllowed, 405),
            (request_timeout(&req), ApiError::RequestTimeout, 408),
            (conflict(&req), ApiError::Conflict(String::new()), 409),
            (
                unprocessable_content(&req),
                ApiError::UnprocessableEntity(String::new()),
                422,
            ),
            (failed_dependency(&req), ApiError::FailedDependency, 424),
            (too_many_requests(&req), ApiError::TooManyRequests, 429),
            (internal_error(), ApiError::InternalError, 500),
            (not_implemented(), ApiError::NotImplemented, 501),
        ];

        for (actual, expected, code) in items {
            assert_matches!(actual.status().code, code);
            assert_matches!(actual, expected);
        }
    }
}
