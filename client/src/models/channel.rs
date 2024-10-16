use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::models::{serialize_naive_date_time, deserialize_naive_date_time};

#[derive(Serialize, Deserialize, Debug)]
pub struct NewChannel{
    name: String,
    server_id: i32,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Channel{
    pub id: i32,
    pub server_id: i32,
    pub name: String,
    #[serde(serialize_with ="serialize_naive_date_time", deserialize_with = "deserialize_naive_date_time")]
    pub created_at: NaiveDateTime,
}
impl Channel {
    pub fn new(
        id: i32,
        name: String,
        server_id: i32,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            name,
            server_id,
            created_at
        }
    }
    pub fn default() -> Self {
        Self::new(
            0,
            "default".to_string(),
            0,
            NaiveDateTime::default()
        )

    }
    pub fn created_at(self: &Self) -> String{
        self.created_at.format("%m/%d/%Y %I:%M %p").to_string()
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MultiChannel {
    pub(crate) channels: Vec<Channel>,
}
impl MultiChannel {
    pub fn new(
        channels: Vec<Channel>,
    ) -> Self {
        Self {
            channels
        }
    }
    pub fn default() -> Self {
        Self::new(
            (1..20).map(|_| Channel::default()).collect()
        )

    }
}