mod error;
mod models;
mod repository;
mod routes;
mod service;

pub(crate) use error::SystemRoleError;
pub(crate) use models::*;
pub(crate) use service::SystemRoleService;

pub(crate) fn get_system_role_routes() -> Vec<rocket::Route> {
    routes![
        routes::create_role,
        routes::list_roles,
        routes::update_role,
        routes::delete_role,
        routes::assign_role,
        routes::assigned_role,
        routes::reset_role,
    ]
}
