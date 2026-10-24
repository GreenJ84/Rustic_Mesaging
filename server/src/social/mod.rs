pub mod error;
pub mod friend;
pub mod request;
pub mod routes;
pub mod service;

pub fn request_routes() -> Vec<rocket::Route> {
    rocket::routes![
        routes::create_request,
        routes::get_request,
        routes::get_friend_requests,
        routes::update_request,
        routes::accept_request,
        routes::deny_request,
        routes::delete_request,
    ]
}

pub fn friend_routes() -> Vec<rocket::Route> {
    rocket::routes![
        routes::get_friend_status,
        routes::get_friend,
        routes::get_friend_previews,
        routes::get_friends,
        routes::update_friend_status,
        routes::remove_friend,
    ]
}
