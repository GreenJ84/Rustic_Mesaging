pub(crate) mod error;
pub(crate) mod models;
pub(crate) mod repository;
pub(crate) mod routes;
pub(crate) mod service;

pub(crate) mod membership;
pub(crate) mod message;

pub fn get_chat_routes() -> Vec<rocket::Route> {
    let mut routes = routes![
        routes::create,
        routes::get,
        routes::update,
        routes::join,
        routes::add_members,
        routes::get_chat_members,
        routes::leave,
    ];
    routes.extend(routes![
        message::routes::create,
        message::routes::get,
        message::routes::update,
        message::routes::delete,
    ]);
    routes
}
