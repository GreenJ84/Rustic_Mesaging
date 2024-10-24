use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use yew::Html;
use crate::comps::icon::{member_avatars, server_icons};
use crate::models::{serialize_naive_date_time, deserialize_naive_date_time};
use crate::models::friend_request::RequestDetailed;

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
    pub fn icon(self, class_name: &str, size: &str) -> Html{
        match self.icon {
            Some(option) => {
                let parts: Vec<&str> = option.split(':').collect();
                server_icons(&class_name, &size, parts[1]).get(parts[0]).unwrap().to_owned()
            }
            None => { server_icons(&class_name, &size, "var(--accent-green)").get("default").unwrap().to_owned() }
        }
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
        Self::new(
            Vec::<Server>::new()
        )

    }
}