use serde::{Deserialize, Serialize};
use yew_router::Routable;
use crate::models::channel::Channel;
use crate::models::member::Member;
use crate::models::message::{Message, MessageThread};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Chat{
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) created_at: chrono::NaiveDateTime
}
impl Chat{
    pub fn default() -> Self{
        Self{
            id: 0,
            name: "Chat".to_string(),
            created_at: Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MultiChatPreview {
    pub(crate) chat_previews: Vec<Chat>,
}
impl MultiChatPreview{
    pub fn default() -> Self {
        Self {
            chat_previews: Vec::<Chat>::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ChatDetail {
    pub(crate) chat: Chat,
    pub(crate) thread: MessageThread
}
impl ChatDetail{
    pub fn default() -> Self {
        Self {
            chat: Chat::default(),
            thread: MessageThread::default(),
        }
    }
}