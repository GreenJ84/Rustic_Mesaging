pub(crate) mod errors;
pub(crate) mod models;
pub(crate) mod repository;
pub(crate) mod routes;
pub(crate) mod service;

pub mod membership;
pub mod channel;

pub fn get_server_routes() -> Vec<rocket::Route> {
    routes![
        routes::create,
        routes::get,
        routes::get_servers_list,
        routes::search_servers,
        routes::update,
        routes::delete,
        routes::join_server,
        routes::get_server_members,
        routes::leave_server,
    ]
}