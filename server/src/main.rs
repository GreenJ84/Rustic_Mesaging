#![allow(unused)]

#[macro_use]
extern crate rocket;
extern crate diesel;
extern crate dotenv;
extern crate tokio;

mod models;
mod db;
mod schema;
mod routes;
mod service;
#[cfg(test)] mod tests;

use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use db::establish_db_connection;
use crate::models::web_socket::WebSocketManager;
use self::routes::*;

#[launch]
fn rocket() -> _ {
    let db_pool = establish_db_connection();
    let web_socket_manager = Arc::new(WebSocketManager::new());

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all()) // Allow all origins
        .allowed_methods(vec!["Head", "Get", "Post", "Put", "Delete"]
             .iter()
             .map(|s| FromStr::from_str(s).unwrap())
             .collect()
        )
        .allowed_headers(AllowedHeaders::all()) // Allow all headers
        .allow_credentials(true)
        .expose_headers(HashSet::from([
            "Authorization".to_string(),
            "Content-Type".to_string(),
        ]))
        .to_cors()
        .expect("error creating CORS options");

    rocket::build()
        .attach(cors)
        .manage(db_pool)
        .manage(web_socket_manager)
        .mount("/", routes![
            auth_route::register, // POST '/register' { Register}
            auth_route::login, // POST '/login' { Login }
        ])
        // All Routes below require JWT member authentication
        .mount("/member", routes![
            member_routes::get, // GET '/member'
            member_routes::update, // PUT '/member' { NewMember }
            auth_route::update_password, // PUT '/password' { Register }
            member_routes::delete, // DELETE '/member'
            member_routes::get_by_username, // GET '/member/<username>'
            member_routes::search_members, // GET '/member/search?<search_term>'
            member_routes::get_all_member_chats, // GET '/member/chats'
            member_routes::get_all_member_servers, // GET '/member/servers'
            member_routes::get_all_friends, // GET '/member/friends'
            member_routes::get_friend_requests, // GET '/member/requests'
        ])
        .mount("/chat", routes![
            member_chat_routes::join_chat, // POST '/chat/<chat_id>/join',
            member_chat_routes::get_membership_status, // GET '/chat/<chat_id>/status',
            member_chat_routes::update_membership_status, // PUT '/chat/<chat_id>/status',
            member_chat_routes::leave_chat, // DELETE '/chat/<chat_id>/leave',

            chat_routes::create, // POST '/chat'  { NewChatWithMembers }
            chat_routes::get, // GET '/chat/<chat_id>'
            chat_routes::update, // PUT '/chat/<chat_id>' { NewChat }
            chat_routes::delete, // DELETE '/chat/<chat_id>'
                chat_routes::get_chat_members, // GET '/chat/<chat_id>/members'
            chat_routes::get_chat_thread, // GET '/chat/<chat_id>/thread'
        ])
        .mount("/message", routes![
            message_routes::create, // POST '/message' { NewMessage }
            message_routes::get, // GET '/message/<message_id>'
            message_routes::update, // PUT '/message/<message_id>' { NewMessage }
            message_routes::delete, // DELETE '/message/<message_id>'
            web_sockets::ws_direct, //  '/message'
        ])
        .mount("/request", routes![
            friend_request_routes::create_request, // POST '/request' { NewFriendRequest }
            friend_request_routes::get_request, // GET '/request/<sender_id>/<receiver_id>'
            friend_request_routes::update_request, // PUT '/request' { NewFriendRequest }
            friend_request_routes::accept_request, // DELETE '/request/<sender_id>/<receiver_id>/accept'
            friend_request_routes::deny_request, // DELETE '/request/<sender_id>/<receiver_id>/deny'
        ])
        .mount("/friend", routes![
            friend_routes::add_friend, // POST '/friend/<friend_id>'
            friend_routes::get_friend_status, // GET '/friend/<friend_id>'
            friend_routes::update_friend_status, // PUT '/friend/<friend_id>'
            friend_routes::remove_friend, // DELETE '/friend/<friend_id>'
        ])
        .mount("/server", routes![
            server_membership_routes::join_server, // POST '/server/<server_id>/join'
            server_membership_routes::get_membership_status, // GET '/server/<server_id>/membership'
            server_membership_routes::update_membership_status, // PUT '/server/<server_id>/membership'
            server_membership_routes::leave_server, // DELETE '/server/<server_id>/leave'

            server_routes::create, // POST '/server' { NewServer } //
            server_routes::get, // GET '/server/<server_id>'
            server_routes::update, // PUT '/server/<server_id>' { NewServer }
            server_routes::delete, // DELETE '/server/<server_id>'
            server_routes::search_servers, // GET '/server/search?<search_term>
            server_routes::get_all_servers, // GET '/server/all'
                server_routes::get_server_members, // GET '/server/<server_id>/members'
            server_routes::get_server_channels, // GET '/server/<server_id>/channels'
        ])
        .mount("/channel", routes![
            channel_routes::create, // POST '/channel' { NewChannel }
            channel_routes::get, // GET '/channel/<channel_id>'
            channel_routes::update, // PUT '/channel' { NewChannel }
            channel_routes::delete, // DELETE '/channel/<channel_id>'
            channel_routes::get_channel_posts, // GET '/channel/<channel_id>/posts'
            web_sockets::channel_websocket, //  '/channel'
        ])
        .mount("/post", routes![
            post_routes::create, // POST '/post' { NewPost }
            post_routes::get, // GET '/post/<post_id>'
            post_routes::update, // PUT '/post/<post_id>' { NewPost }
            post_routes::delete, // DELETE '/post/<post_id>'
        ])
        .mount("/reports", routes![
            report_routes::member_activity_report,
            report_routes::server_membership_report
        ])
        .register("/", catchers![
            auth_route::unauthorized_catcher
        ])

}
