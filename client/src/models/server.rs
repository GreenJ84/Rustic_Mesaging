use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::models::{serialize_naive_date_time, deserialize_naive_date_time};


#[derive(Serialize, Deserialize, Debug)]
pub struct NewServer{
    name: String,
    description: String,
    icon: Option<String>,
    owner_id: i32,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Server{
    pub id: i32,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub owner_id: i32,
    #[serde(serialize_with ="serialize_naive_date_time", deserialize_with = "deserialize_naive_date_time")]
    pub created_at: NaiveDateTime,
}
impl Server {
    pub fn new(
        id: i32,
        name: String,
        description: String,
        icon: Option<String>,
        owner_id: i32,
        created_at: NaiveDateTime
    ) -> Self {
        Self {
            id,
            name,
            description,
            icon,
            owner_id,
            created_at
        }
    }
    pub fn default() -> Self {
        Self::new(
            0,
            "default".to_string(),
            "I is Server".to_string(),
            None,
            0,
            NaiveDateTime::default()
        )

    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MultiServer {
    pub(crate) servers: Vec<Server>,
}
impl MultiServer {
    pub fn new(
        servers: Vec<Server>,
    ) -> Self {
        Self {
            servers
        }
    }
    pub fn default() -> Self {
        let servers = (1..25).map(|_| {
            Server::default()
        }).collect::<Vec<Server>>();
        Self::new(
            servers
        )

    }
}