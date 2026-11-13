//! HTTP boundary types, CORS policy, and Rocket error handling.

mod catchers;
mod cors;
mod error;

pub(crate) use cors::create_cors;
pub(crate) use error::ApiError;

/// Returns all framework-level error catchers used by the application.
pub(crate) fn get_catchers() -> Vec<rocket::Catcher> {
    catchers![
        catchers::bad_request,
        catchers::unauthorized,
        catchers::forbidden,
        catchers::not_found,
        catchers::method_not_allowed,
        catchers::request_timeout,
        catchers::conflict,
        catchers::unprocessable_content,
        catchers::failed_dependency,
        catchers::too_many_requests,
        catchers::internal_error,
        catchers::not_implemented
    ]
}
