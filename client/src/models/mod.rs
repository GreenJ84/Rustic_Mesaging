use chrono::NaiveDateTime;
use serde::{de, Deserialize, Deserializer, Serializer};

pub(crate) mod member;
pub(crate) mod server;
pub(crate) mod channel;
pub(crate) mod message;
pub(crate) mod post;
pub(crate) mod server_membership;
pub(crate) mod friends;
pub(crate) mod friend_request;
pub(crate) mod chat;
pub(crate) mod report;

pub fn serialize_naive_date_time<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&dt.format("%Y-%m-%dT%H:%M:%S").to_string())
}

fn deserialize_naive_date_time<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f")
        .map_err(de::Error::custom)
}