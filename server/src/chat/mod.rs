pub(crate) mod error;
pub(crate) mod models;
pub(crate) mod repository;
pub(crate) mod routes;
pub(crate) mod service;

pub(crate) mod membership;

pub fn get_chat_routes() -> Vec<rocket::Route> {
    routes![
        routes::create,
        routes::get,
        routes::update,
        routes::join,
        routes::add_members,
        routes::get_chat_members,
        routes::leave,
    ]
}
