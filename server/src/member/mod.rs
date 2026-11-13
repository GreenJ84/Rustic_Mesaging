pub(super) mod error;
pub(crate) mod models;
pub(crate) mod repository;
pub(crate) mod routes;
pub(crate) mod service;

pub(crate) fn get_member_routes() -> Vec<rocket::Route> {
    let mut routes = routes::routes();
    routes.extend(routes![crate::server::routes::get_member_servers]);
    routes
}