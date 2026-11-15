pub(in crate::server) mod error;
pub(in crate::server) mod models;
pub(in crate::server) mod repository;
pub(in crate::server) mod routes;
pub(crate) mod service;

pub(in crate::server) fn get_post_routes() -> Vec<rocket::Route> {
    routes::routes()
}
