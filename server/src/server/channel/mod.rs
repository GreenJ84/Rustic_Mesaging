pub(in crate::server) mod error;
pub(in crate::server) mod models;
pub(in crate::server) mod repository;
pub(in crate::server) mod routes;
pub(crate) mod service;

pub(crate) mod post;

pub(in crate::server) fn get_channel_routes() -> Vec<rocket::Route> {
    let mut routes = routes::routes();
    routes.extend(post::get_post_routes());
    routes
}
