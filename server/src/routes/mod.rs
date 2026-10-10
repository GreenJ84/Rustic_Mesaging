use rocket::response::status;

use crate::api_error::ApiError;

pub(crate) mod catchers;
pub(crate) mod channel_routes;
pub(crate) mod friend_request_routes;
pub(crate) mod friend_routes;
pub(crate) mod member_chat_routes;
pub(crate) mod message_routes;
pub(crate) mod post_routes;
pub(crate) mod report_routes;
pub(crate) mod server_membership_routes;
pub(crate) mod web_sockets;

pub(crate) type ApiResponse<T> = Result<status::Custom<T>, ApiError>;

pub(crate) type CustomResponse<T> = Result<status::Custom<T>, status::Custom<String>>;
