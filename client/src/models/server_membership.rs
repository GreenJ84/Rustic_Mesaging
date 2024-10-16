use serde::{Deserialize, Serialize};
use crate::models::deserialize_naive_date_time;


#[derive(Serialize, Deserialize, Debug)]
pub struct NewServerMembership {
    pub server_id: i32,
    pub member_id: i32,
}
#[derive(Deserialize, Debug)]
pub struct ServerMembership {
    pub id: i32,
    pub server_id: i32,
    pub member_id: i32,
    #[serde(deserialize_with = "deserialize_naive_date_time")]
    pub joined_at: chrono::NaiveDateTime,
}