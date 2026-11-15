pub mod error;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;

pub mod post;

pub(in crate::server) fn get_channel_routes() -> Vec<rocket::Route> {
    let mut routes = routes::routes();
    routes.extend(post::get_post_routes());
    routes
}
