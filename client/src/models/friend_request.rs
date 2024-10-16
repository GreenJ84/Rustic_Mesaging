use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::models::member::Member;

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
    fn reqs() -> Vec<RequestDetailed> {
        (1..15).into_iter()
            .map(|_| RequestDetailed::default())
            .collect()
    }
    Self {
        incoming: reqs().clone(),
        outgoing: reqs(),
    }
}
}