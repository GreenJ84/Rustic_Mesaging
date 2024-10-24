use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::models::channel::Channel;
use crate::models::member::Member;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FriendRequest{
    pub(crate) sender_id: i32,
    pub(crate) receiver_id: i32,
    pub(crate) note: Option<String>,
    pub(crate) created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RequestDetailed {
    pub(crate) member: Member,
    pub(crate) note: Option<String>,
    pub(crate) created_at: NaiveDateTime,
}

impl RequestDetailed{
    pub fn default() -> Self{
        Self {
            member: Member::default(),
            note: Some("Friend Request".to_string()),
            created_at: NaiveDateTime::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MultiFriendRequest {
    pub(crate) incoming: Vec<RequestDetailed>,
    pub(crate) outgoing: Vec<RequestDetailed>,
}
impl MultiFriendRequest{
pub fn default() -> Self{
    Self {
        incoming: Vec::<RequestDetailed>::new(),
        outgoing: Vec::<RequestDetailed>::new(),
    }
}
}