use crate::comps::icon::member_avatars;
use crate::models::friend_request::RequestDetailed;
use crate::models::{deserialize_naive_date_time, serialize_naive_date_time};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use yew::{html, Html};

use super::{AvatarRendering, ContextDefault};

pub(crate) use shared::models::member::{
    ChangePasswordInput, MemberDetailed as Member, MemberShort, MemberUpdateInput,
    MultiMemberPreview as MultiMember, NewMemberInput,
};

impl ContextDefault for Member {
    fn default() -> Self {
        Self {
            id: 0,
            username: "guest".to_string(),
            email: "member.email@provider".to_string(),
            avatar: String::from("default"),
            created_at: NaiveDateTime::default(),
        }
    }
}
impl AvatarRendering for Member {
    fn avatar(self, class_name: &str, size: &str) -> Html {
        member_avatars(&class_name, &size)
            .get(&self.avatar)
            .unwrap()
            .to_owned()
    }
}

impl ContextDefault for MemberShort {
    fn default() -> Self {
        Self {
            id: 0,
            username: "guest".to_string(),
            avatar: String::from("default"),
        }
    }
}
impl AvatarRendering for MemberShort {
    fn avatar(self, class_name: &str, size: &str) -> Html {
        member_avatars(&class_name, &size)
            .get(&self.avatar)
            .unwrap()
            .to_owned()
    }
}

impl ContextDefault for MultiMember {
    fn default() -> Self {
        Self {
            members: Vec::<MemberShort>::new(),
        }
    }
}
