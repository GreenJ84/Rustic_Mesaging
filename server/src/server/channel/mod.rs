pub mod error;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;

pub fn get_channel_routes() -> Vec<rocket::Route> {
    routes![
        routes::create,
        routes::get,
        routes::get_server_channels,
        routes::get_server_channel_previews,
        routes::update,
        routes::delete,
    ]
}