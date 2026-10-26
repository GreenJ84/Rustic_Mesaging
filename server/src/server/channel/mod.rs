pub mod error;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;

pub mod post;

pub fn get_channel_routes() -> Vec<rocket::Route> {
    let mut routes = routes![
        routes::create,
        routes::get,
        routes::get_server_channels,
        routes::get_server_channel_previews,
        routes::update,
        routes::delete,
    ];
    routes.extend(post::get_post_routes());
    routes
}
