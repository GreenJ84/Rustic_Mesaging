use crate::comps::icon::member_avatars;
use crate::models::friend_request::RequestDetailed;
use crate::models::{deserialize_naive_date_time, serialize_naive_date_time};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use yew::{html, Html};

pub(crate) use shared::member::NewMemberInput;

pub(crate) use shared::member::ChangePasswordInput;

pub(crate) use shared::member::MemberUpdateInput;

pub(crate) use shared::member::MemberProfile as Member;
impl MemberDefaults for Member {
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

pub(crate) use shared::member::MemberShort;
impl MemberDefaults for MemberShort {
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

pub(crate) use shared::member::MultiMemberPreview as MultiMember;
impl MemberDefaults for MultiMember {
    fn default() -> Self {
        Self {
            members: Vec::<MemberShort>::new(),
        }
    }
}

pub(crate) trait MemberDefaults {
    fn default() -> Self;
}
pub(crate) trait AvatarRendering {
    fn avatar(self, class_name: &str, size: &str) -> Html;
}
