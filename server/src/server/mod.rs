pub(crate) mod errors;
pub(crate) mod models;
pub(crate) mod repository;
pub(crate) mod routes;
pub(crate) mod service;

pub mod channel;
pub mod membership;

pub fn get_server_routes() -> Vec<rocket::Route> {
    let mut routes = routes![
        routes::create,
        routes::get,
        routes::get_servers_list,
        routes::search_servers,
        routes::update,
        routes::delete,
        routes::join_server,
        routes::get_server_members,
        routes::leave_server,
    ];
    routes.extend(channel::get_channel_routes());
    routes
}
