pub mod error;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;

pub fn get_post_routes() -> Vec<rocket::Route> {
    routes![
        routes::create,
        routes::get,
        routes::get_page,
        routes::update,
        routes::delete,
    ]
}
