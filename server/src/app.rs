use std::collections::HashSet;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use rocket::{http, Build, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions, Method};

use crate::cors;
use crate::database::Database;
use crate::id_generator::IdGenerator;
use crate::schema::server;

use crate::auth::system_roles::get_system_role_routes;
use crate::auth::{get_auth_routes, service::AuthService};
// use crate::chat::routes as chat_routes;
use crate::member::{get_member_routes, service::MemberService};
use crate::server::channel::service::ChannelService;
use crate::server::{get_server_routes, service::ServerService};
use crate::social::{get_friend_routes, get_request_routes, service::SocialService};
// use crate::server::routes as server_routes;

use crate::config::AppConfig;

pub fn build() -> Rocket<Build> {
    let config = AppConfig::load().expect("application configuration must be valid");
    let database = Database::new(&config.database_url)
        .map_err(|e| {
            log::error!("Failed to connect to the database: {:?}", e);
            e
        })
        .expect("Database connection must be established");
    let id_generator = IdGenerator::new();

    let auth_service = AuthService::new(
        database.clone(),
        id_generator,
        config.jwt_secret.as_bytes(),
        config.cookie_secure,
    );
    let member_service = MemberService::new(database.clone());
    let social_service = SocialService::new(database.clone());
    let server_service = ServerService::new(database.clone());
    let channel_service = ChannelService::new(database.clone());

    let cors = cors::create_cors(&config.allowed_origins)
        .to_cors()
        .expect("error creating CORS options");

    rocket::build()
        .attach(cors)
        .manage(auth_service)
        .manage(member_service)
        .manage(social_service)
        .manage(server_service)
        .manage(channel_service)
        .mount("/", get_auth_routes())
        .mount("/roles", get_system_role_routes())
        .mount("/member", get_member_routes())
        .mount("/friends", get_friend_routes())
        .mount("/requests", get_request_routes())
        .mount("/servers", get_server_routes())
}

// use crate::models::web_socket::WebSocketManager;

// #[rocket::main]
// async fn main() -> Result<(), rocket::Error> {
//     let rocket = rocket::build().mount("/", routes![index]);

//     // Start the Socket.IO server in a separate Tokio task
//     tokio::spawn(async {
//         // Replace this with your actual Socket.IO server initialization
//         let socket_io_server = some_socket_io_library::Server::new();
//         socket_io_server.run().await.unwrap();
//     });

//     // Launch Rocket
//     rocket.launch().await
// }

// #[launch]
// fn rocket() -> _ {
//     //! Depreciating
//     // TODO: Implement Websockets
//     let web_socket_manager = Arc::new(WebSocketManager::new());

//     rocket::build()
//         .manage(web_socket_manager)
//         .mount(
//             "/",
//             routes![
//                 auth_routes::register,
//                 auth_routes::login,
//                 auth_routes::refresh,
//                 auth_routes::logout,
//                 auth_routes::logout_all,
//                 auth_routes::update_password,
//             ],
//         )
//         .mount("/roles", system_roles::routes())
//         .mount(
//             "/member",
//             routes![
//                 member_routes::head,
//                 member_routes::get,
//                 member_routes::update,
//                 member_routes::delete,
//                 member_routes::get_by_username,
//                 member_routes::search_members,
//                 member_routes::get_all_member_chats,
//                 member_routes::get_all_member_servers,
//                 member_routes::get_all_friends,
//                 member_routes::get_friend_requests,
//             ],
//         )
// .mount(
//     "/chat",
//     routes![
//         member_chat_routes::join_chat, // POST '/chat/<chat_id>/join',
//         member_chat_routes::get_membership_status, // GET '/chat/<chat_id>/status',
//         member_chat_routes::update_membership_status, // PUT '/chat/<chat_id>/status',
//         member_chat_routes::leave_chat, // DELETE '/chat/<chat_id>/leave',
//         chat_routes::create,           // POST '/chat'  { NewChatWithMembers }
//         chat_routes::get,              // GET '/chat/<chat_id>'
//         chat_routes::update,           // PUT '/chat/<chat_id>' { NewChat }
//         chat_routes::delete,           // DELETE '/chat/<chat_id>'
//         chat_routes::get_chat_members, // GET '/chat/<chat_id>/members'
//         chat_routes::get_chat_thread,  // GET '/chat/<chat_id>/thread'
//     ],
// )
// .mount(
//     "/message",
//     routes![
//         message_routes::create, // POST '/message' { NewMessage }
//         message_routes::get,    // GET '/message/<message_id>'
//         message_routes::update, // PUT '/message/<message_id>' { NewMessage }
//         message_routes::delete, // DELETE '/message/<message_id>'
//         web_sockets::ws_direct, //  '/message'
//     ],
// )
// .mount(
//     "/request",
//     routes![
//         friend_request_routes::create_request, // POST '/request' { NewFriendRequest }
//         friend_request_routes::get_request,    // GET '/request/<sender_id>/<receiver_id>'
//         friend_request_routes::update_request, // PUT '/request' { NewFriendRequest }
//         friend_request_routes::accept_request, // DELETE '/request/<sender_id>/<receiver_id>/accept'
//         friend_request_routes::deny_request, // DELETE '/request/<sender_id>/<receiver_id>/deny'
//     ],
// )
// .mount(
//     "/friend",
//     routes![
//         friend_routes::add_friend,           // POST '/friend/<friend_id>'
//         friend_routes::get_friend_status,    // GET '/friend/<friend_id>'
//         friend_routes::update_friend_status, // PUT '/friend/<friend_id>'
//         friend_routes::remove_friend,        // DELETE '/friend/<friend_id>'
//     ],
// )
// .mount(
//     "/server",
//     routes![
//         server_membership_routes::join_server, // POST '/server/<server_id>/join'
//         server_membership_routes::get_membership_status, // GET '/server/<server_id>/membership'
//         server_membership_routes::update_membership_status, // PUT '/server/<server_id>/membership'
//         server_membership_routes::leave_server, // DELETE '/server/<server_id>/leave'
//         server_routes::create,                  // POST '/server' { NewServer } //
//         server_routes::get,                     // GET '/server/<server_id>'
//         server_routes::update,                  // PUT '/server/<server_id>' { NewServer }
//         server_routes::delete,                  // DELETE '/server/<server_id>'
//         server_routes::search_servers,          // GET '/server/search?<search_term>
//         server_routes::get_all_servers,         // GET '/server/all'
//         server_routes::get_server_members,      // GET '/server/<server_id>/members'
//         server_routes::get_server_channels,     // GET '/server/<server_id>/channels'
//     ],
// )
// .mount(
//     "/channel",
//     routes![
//         channel_routes::create,            // POST '/channel' { NewChannel }
//         channel_routes::get,               // GET '/channel/<channel_id>'
//         channel_routes::update,            // PUT '/channel' { NewChannel }
//         channel_routes::delete,            // DELETE '/channel/<channel_id>'
//         channel_routes::get_channel_posts, // GET '/channel/<channel_id>/posts'
//         web_sockets::channel_websocket,    //  '/channel'
//     ],
// )
// .mount(
//     "/post",
//     routes![
//         post_routes::create, // POST '/post' { NewPost }
//         post_routes::get,    // GET '/post/<post_id>'
//         post_routes::update, // PUT '/post/<post_id>' { NewPost }
//         post_routes::delete, // DELETE '/post/<post_id>'
//     ],
// )
// .mount(
//     "/reports",
//     routes![
//         report_routes::member_activity_report,
//         report_routes::download_member_activity_report,
//         report_routes::server_membership_report,
//         report_routes::download_server_membership_report
//     ],
// )
// .register(
//     "/",
//     catchers![
//         catchers::unauthorized_catcher,
//         catchers::not_found,
//         catchers::conflict_catcher,
//         catchers::failed_dependency,
//         catchers::internal_error
//     ],
// )
// }
