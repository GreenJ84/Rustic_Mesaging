pub mod error;
pub mod guards;
pub mod models;
pub mod password;
pub mod routes;
pub mod service;

pub(crate) mod session;
pub(crate) mod system_roles;
pub(crate) use guards::*;

pub fn get_auth_routes() -> Vec<rocket::Route> {
    routes![
        routes::register,
        routes::login,
        routes::refresh,
        routes::logout,
        routes::logout_all,
        routes::update_password,
    ]
}
