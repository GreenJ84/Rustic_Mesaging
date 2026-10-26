use chrono::NaiveDateTime;
use diesel::{AsChangeset, Associations, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::member::models::{Member, MemberId, MemberShort};
use crate::schema::post;
use crate::server::channel::models::{Channel, ChannelId};
use crate::utils::{Page, Validator};

use super::error::PostError;

pub type PostId = i32;

#[derive(Debug, FromForm)]
pub struct NewPostInput {
    content: String,
}

#[derive(Insertable)]
#[diesel(table_name = post)]
pub(crate) struct NewPost {
    content: String,
    author_id: MemberId,
    channel_id: ChannelId,
}
impl NewPost {
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn author_id(&self) -> MemberId {
        self.author_id
    }
    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }
}
impl TryFrom<(MemberId, ChannelId, NewPostInput)> for NewPost {
    type Error = PostError;
    fn try_from(
        (member_id, channel_id, input): (MemberId, ChannelId, NewPostInput),
    ) -> Result<Self, Self::Error> {
        let content = Validator::validate_text(input.content, None, Some(500)).map_err(|e| {
            PostError::InvalidInput(format!("Post content {}", e.message().to_string()))
        })?;
        Ok(Self {
            content,
            author_id: member_id,
            channel_id: channel_id,
        })
    }
}

#[derive(Debug, FromForm)]
pub struct PostChangesetInput {
    content: Option<String>,
}
impl PostChangesetInput {
    pub fn is_empty(&self) -> bool {
        self.content.is_none()
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = post)]
pub(crate) struct PostChangeset {
    content: Option<String>,
}
impl PostChangeset {
    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }
}
impl TryFrom<PostChangesetInput> for PostChangeset {
    type Error = PostError;
    fn try_from(input: PostChangesetInput) -> Result<Self, Self::Error> {
        if input.is_empty() {
            return Err(PostError::InvalidChangeset(
                "No fields provided for update".into(),
            ));
        }
        let content = if let Some(content) = input.content {
            let validated = Validator::validate_text(content, None, Some(500)).map_err(|e| {
                PostError::InvalidChangeset(format!("Post content {}", e.message().to_string()))
            })?;
            Some(validated)
        } else {
            None
        };
        Ok(Self { content })
    }
}

#[derive(Identifiable, Queryable, Selectable, Associations, Serialize)]
#[diesel(table_name = post)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Channel, foreign_key = channel_id))]
#[diesel(belongs_to(Member, foreign_key = author_id))]
pub struct Post {
    id: PostId,
    content: String,
    author_id: MemberId,
    channel_id: ChannelId,
    created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostDetail {
    id: PostId,
    content: String,
    author: MemberShort,
    channel_id: ChannelId,
    created_at: NaiveDateTime,
}
impl PostDetail {
    pub fn new(
        id: PostId,
        content: String,
        author: MemberShort,
        channel_id: ChannelId,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            content,
            author,
            channel_id,
            created_at,
        }
    }
}
impl From<(Post, MemberShort)> for PostDetail {
    fn from((post, author): (Post, MemberShort)) -> Self {
        Self {
            id: post.id,
            content: post.content,
            author: author,
            channel_id: post.channel_id,
            created_at: post.created_at,
        }
    }
}

pub type MultiPost = Vec<PostDetail>;
pub type PostPage = Page<MultiPost>;
