use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::member::{MemberId, MemberShort};
use crate::models::server::channel::channel::ChannelId;

pub type PostId = i32;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewPostInput {
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PostChangesetInput {
    pub content: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Post {
    pub id: PostId,
    pub content: String,
    pub author_id: MemberId,
    pub channel_id: ChannelId,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PostDetailed {
    pub id: PostId,
    pub content: String,
    pub author: MemberShort,
    pub channel_id: ChannelId,
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PostList<T> {
    pub posts: Vec<T>,
}

pub type MultiPost = PostList<Post>;
pub type MultiPostDetailed = PostList<PostDetailed>;

#[cfg(test)]
mod tests {
    use super::{NewPostInput, PostChangesetInput, PostList};

    #[test]
    fn creation_input_contains_only_client_owned_content() {
        let value = serde_json::to_value(NewPostInput {
            content: "hello".into(),
        })
        .unwrap();

        assert_eq!(value, serde_json::json!({ "content": "hello" }));
    }

    #[test]
    fn changeset_and_collection_shapes_are_stable() {
        let changeset = serde_json::to_value(PostChangesetInput { content: None }).unwrap();
        let list =
            serde_json::to_value(PostList::<serde_json::Value> { posts: Vec::new() }).unwrap();

        assert!(changeset["content"].is_null());
        assert_eq!(list, serde_json::json!({ "posts": [] }));
    }
}
