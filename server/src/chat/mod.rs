pub(crate) mod error;
pub(crate) mod models;
pub(crate) mod repository;
pub(crate) mod routes;
pub(crate) mod service;

pub(crate) mod message;
pub(crate) mod participant;

pub fn get_chat_routes() -> Vec<rocket::Route> {
    let mut routes = routes::routes();
    routes.extend(message::routes::routes());
    routes
}
