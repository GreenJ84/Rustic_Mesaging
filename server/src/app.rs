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
use crate::utils::get_catchers;

use crate::auth::system_roles::get_system_role_routes;
use crate::auth::{get_auth_routes, service::AuthService};
// use crate::chat::routes as chat_routes;
use crate::chat::message::service::MessageService;
use crate::chat::service::ChatService;
use crate::member::{get_member_routes, service::MemberService};
use crate::server::channel::post::service::PostService;
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
    let chat_service = ChatService::new(database.clone());
    let message_service = MessageService::new(database.clone());
    let post_service = PostService::new(database.clone());

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
        .manage(chat_service)
        .manage(message_service)
        .manage(post_service)
        .mount("/", get_auth_routes())
        .mount("/roles", get_system_role_routes())
        .mount("/member", get_member_routes())
        .mount("/friends", get_friend_routes())
        .mount("/requests", get_request_routes())
        .mount("/servers", get_server_routes())
        .mount("/chat", crate::chat::get_chat_routes())
        .register("/", get_catchers())
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
