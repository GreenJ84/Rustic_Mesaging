pub mod error;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;

pub(in crate::server) fn get_post_routes() -> Vec<rocket::Route> {
    routes::routes()
}
