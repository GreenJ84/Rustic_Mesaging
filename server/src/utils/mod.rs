mod pagination;
mod validation;
mod catchers;

pub use pagination::{Page, Paginator};
pub use validation::{SearchValidation, ValidationError, Validator};

pub fn get_catchers() -> Vec<rocket::Catcher> {
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