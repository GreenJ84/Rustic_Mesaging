use std::io::ErrorKind;
use ws::{WebSocket};
use rocket::{State};
use rocket::futures::{StreamExt};
use rocket::http::Status;
use rocket::response::status;

use crate::db::{DbPool, get_db_connection};
use crate::models::message::{Message, NewMessage};
use crate::models::post::NewPost;
use crate::models::web_socket::{WebSocketId, WebSocketMgr};
use crate::service::CrudOps;
use crate::service::message_service::MessageService;
use crate::service::post_service::PostService;

#[get("/ws/<channel_id>/<user_id>")]
pub fn channel_websocket<'a>(
    channel_id: i32,
    user_id: i32,
    ws: WebSocket,
    manager: &'a State<WebSocketMgr>,
    pool: &'a State<DbPool>
) -> ws::Channel<'a>{

    ws.channel(move |stream| {
        let manager = manager;
        let mut conn = get_db_connection(pool).map_err(|_| {
            return Err::<status::Custom<Message>, status::Custom<String>>(status::Custom(Status::InternalServerError, String::from("Database connection error")))
        }).unwrap();

        Box::pin(async move {
            let (sender, mut receiver) = stream.split();
            let id = WebSocketId::Channel(channel_id);

            manager.add_connection(&id, user_id, sender)
                .await
                .unwrap();

            while let Some(message) = receiver.next().await {
                match message {
                    Ok(msg) => {
                        let new_post = serde_json::from_str::<NewPost>(&msg.into_text()?)
                            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
                            .unwrap();
                        let post = PostService::create(&mut conn, new_post)
                            .map_err(|e| std::io::Error::new(ErrorKind::BrokenPipe, e))
                            .unwrap();
                        manager.broadcast(&id, post).await;
                    }
                    Err(e) => { std::io::Error::new(ErrorKind::BrokenPipe, e); }
                }
            }

            manager.remove_connection(id, user_id).await;

            Ok(())
        })
    })
}

#[get("/ws/<sender_id>/<receiver_id>")]
pub async fn ws_direct<'a>(
    sender_id: i32,
    receiver_id: i32,
    ws: WebSocket,
    manager: &'a State<WebSocketMgr>,
    pool: &'a State<DbPool>
) -> ws::Channel<'a> {

    ws.channel(move |stream| {
        let mut conn = get_db_connection(pool).map_err(|_| {
            return Err::<status::Custom<Message>, status::Custom<String>>(status::Custom(Status::InternalServerError, String::from("Database connection error")))
        }).unwrap();

        Box::pin(async move {
            let (sender, mut receiver) = stream.split();
            let direct_message_id = if sender_id < receiver_id {
                format!("{}_{}", sender_id, receiver_id)
            } else {
                format!("{}_{}", receiver_id, sender_id)
            };
            let id = WebSocketId::DirectMessage(direct_message_id);

            manager.add_connection(&id, sender_id, sender)
                .await
                .unwrap();

            while let Some(message) = receiver.next().await {
                match message {
                    Ok(msg) => {
                        // Deserialize the message and save to the database
                        let new_message = serde_json::from_str::<NewMessage>(&msg.into_text()?)
                            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
                            .unwrap();

                        let message = MessageService::create(&mut conn, new_message)
                            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
                            .unwrap();

                        // Send back a confirmation
                        manager.broadcast(&id, message).await;
                    }
                    Err(e) => eprintln!("Error receiving message: {}", e),
                }
            }
            Ok(())
        })
    })
}