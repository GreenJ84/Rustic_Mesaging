pub(crate) mod errors;
pub(crate) mod models;
pub(crate) mod repository;
pub(crate) mod routes;
pub(crate) mod service;

pub(crate) mod channel;
pub(crate) mod membership;

pub(crate) fn get_server_routes() -> Vec<rocket::Route> {
    let mut routes = routes::routes();
    routes.extend(channel::get_channel_routes());
    routes
}
