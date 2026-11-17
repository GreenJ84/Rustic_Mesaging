mod error;
pub(crate) mod friend;
mod request;
mod routes;
pub(crate) mod service;

pub(crate) fn get_request_routes() -> Vec<rocket::Route> {
    routes::request_routes()
}

pub(crate) fn get_friend_routes() -> Vec<rocket::Route> {
    routes::friend_routes()
}
