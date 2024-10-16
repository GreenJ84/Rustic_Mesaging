use std::collections::HashMap;
use std::sync::{Arc};
use rocket::futures::SinkExt;
use rocket::futures::stream::SplitSink;
use rocket::tokio::sync::{broadcast, Mutex};
use serde::Serialize;
use ws::Message;
use ws::stream::DuplexStream;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum WebSocketId {
    Channel(i32),
    DirectMessage(String),
}

// #[derive(Clone)]
struct WebSocketConnection {
    user_id: i32,
    stream: SplitSink<DuplexStream, Message>,
}

pub type WebSocketMgr = Arc<WebSocketManager>;
#[derive(Clone)]
pub struct WebSocketManager {
    connections: Arc<Mutex<HashMap<WebSocketId, Vec<WebSocketConnection>>>>, // i32 is the channel_id
    sender: Arc<Mutex<broadcast::Sender<String>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            sender: Arc::new(Mutex::new(sender)),
        }
    }

    // Add a new WebSocket connection to a specific channel
    pub async fn add_connection(&self, id: &WebSocketId, user_id: i32, ws: SplitSink<DuplexStream, Message>) -> Result<(), String> {
        let mut connections = self.connections.lock().await;
        let entry = connections.entry(id.clone()).or_insert_with(Vec::new);

        // Check if user already has a connection in the channel
        if entry.iter().any(|conn| conn.user_id == user_id) {
            return Err("User already connected in this channel".to_string());
        }

        entry.push(WebSocketConnection { user_id, stream: ws });
        Ok(())
    }

    // Remove a WebSocket connection from a specific channel
    pub async fn remove_connection(&self, id: WebSocketId, user_id: i32) {
        let mut connections = self.connections.lock().await;
        if let Some(conns) = connections.get_mut(&id) {
            conns.retain(|conn| conn.user_id != user_id);
            if conns.is_empty() {
                connections.remove(&id);
            }
        }
    }

    // Broadcast a message to all connections in a specific channel
    pub async fn broadcast<T: Serialize>(
        &self,
        id: &WebSocketId,
        message: T
    ) {
        let mut connections = self.connections.lock().await;
        if let Some(conns) = connections.get_mut(&id) {
            for conn in conns.iter_mut() {
                let _ = conn.stream.send(ws::Message::Text(serde_json::to_string(&message).unwrap())).await;
            }
        }
    }
}