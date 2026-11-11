use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use yew::Properties;

use super::member::{Member, MemberShort};
use super::{AvatarRendering, ContextDefault};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FullFriend {
    pub(crate) member: MemberShort,
    pub(crate) created_at: NaiveDateTime,
}
impl FullFriend {
    pub fn default() -> Self {
        Self {
            member: MemberShort::default(),
            created_at: NaiveDateTime::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Properties)]
pub struct MultiFriend {
    pub(crate) friends: Vec<FullFriend>,
}
impl MultiFriend {
    pub fn default() -> Self {
        Self {
            friends: Vec::<FullFriend>::new(),
        }
    }
}
