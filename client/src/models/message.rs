use std::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::models::member::{Member, MemberShort};
use crate::models::{serialize_naive_date_time, deserialize_naive_date_time};
use crate::models::chat::Chat;
use crate::models::friend_request::RequestDetailed;

#[derive(Serialize, Deserialize, Debug)]
pub struct NewMessage{
    content: String,
    sender_id: i32,
    chat_id: i32,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message{
    pub(crate) id: i32,
    pub(crate) content: String,
    pub(crate) sender: MemberShort,
    #[serde(serialize_with ="serialize_naive_date_time", deserialize_with = "deserialize_naive_date_time")]
    pub(crate) created_at: chrono::NaiveDateTime,
}
impl Message {
    pub fn default() -> Self{
        Self{
            id: 0,
            content: "Message".to_string(),
            sender: MemberShort::default(),
            created_at: Default::default()
        }
    }
    pub fn created_at(self: &Self) -> String{
        self.created_at.format("%m/%d/%Y %I:%M %p").to_string()
    }
}
impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Message ID: {}, Content: {}, Sender: {:?}, Created At: {}",
            self.id,
            self.content,
            self.sender,
            self.created_at.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MessageThread {
    pub(crate) thread: Vec<Message>,
}
impl MessageThread{
    pub fn default() -> Self {
        Self {
            thread: Vec::<Message>::new(),
        }
    }
}
