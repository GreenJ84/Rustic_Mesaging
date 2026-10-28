pub(super) mod error;
pub(crate) mod models;
pub(crate) mod repository;
pub(crate) mod routes;
pub(crate) mod service;

pub fn get_member_routes() -> Vec<rocket::Route> {
    routes![
        routes::head,
        routes::get,
        routes::get_by_username,
        routes::search_members,
        routes::update,
        routes::delete,
        crate::server::routes::get_member_servers,
    ]
}
