mod catchers;
mod cors;
mod error;

pub(crate) use error::ApiError;
pub(crate) use cors::create_cors;

pub(crate) fn get_catchers() -> Vec<rocket::Catcher> {
    catchers![
        catchers::unauthorized,
        catchers::forbidden,
        catchers::not_found,
        catchers::method_not_allowed,
        catchers::request_timeout,
        catchers::conflict,
        catchers::failed_dependency,
        catchers::internal_error,
    ]
}
