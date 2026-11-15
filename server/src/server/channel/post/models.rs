use chrono::NaiveDateTime;
use diesel::{AsChangeset, Associations, Identifiable, Insertable, Queryable, Selectable};

use crate::member::models::{MemberRow, MemberShortRow};
use crate::schema::post;
use crate::server::channel::models::ChannelRow;
use crate::utils::Validator;
use shared::models::member::{MemberId, MemberShort};
use shared::models::server::channel::channel::ChannelId;
use shared::models::server::channel::post::{
    NewPostInput, Post, PostChangesetInput, PostDetailed, PostId,
};

use super::error::PostError;

#[derive(Insertable)]
#[diesel(table_name = post)]
pub(in crate::server) struct NewPost {
    content: String,
    author_id: MemberId,
    channel_id: ChannelId,
}
impl TryFrom<(MemberId, ChannelId, NewPostInput)> for NewPost {
    type Error = PostError;

    fn try_from(
        (member_id, channel_id, input): (MemberId, ChannelId, NewPostInput),
    ) -> Result<Self, Self::Error> {
        let content =
            Validator::validate_text(&input.content, None, Some(500)).map_err(|error| {
                PostError::InvalidInput(format!("Post content {}", error.message()))
            })?;
        Ok(Self {
            content,
            author_id: member_id,
            channel_id,
        })
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = post)]
pub(in crate::server) struct PostChangeset {
    content: Option<String>,
}
impl TryFrom<PostChangesetInput> for PostChangeset {
    type Error = PostError;

    fn try_from(input: PostChangesetInput) -> Result<Self, Self::Error> {
        let Some(content) = input.content else {
            return Err(PostError::InvalidChangeset(
                "No fields provided for update".into(),
            ));
        };
        let content = Validator::validate_text(&content, None, Some(500)).map_err(|error| {
            PostError::InvalidChangeset(format!("Post content {}", error.message()))
        })?;
        Ok(Self {
            content: Some(content),
        })
    }
}

#[derive(Associations, Identifiable, Queryable, Selectable)]
#[diesel(table_name = post)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(ChannelRow, foreign_key = channel_id))]
#[diesel(belongs_to(MemberRow, foreign_key = author_id))]
pub(in crate::server) struct PostRow {
    id: PostId,
    content: String,
    author_id: MemberId,
    channel_id: ChannelId,
    created_at: NaiveDateTime,
}
impl PostRow {
    pub(in crate::server) fn id(&self) -> PostId {
        self.id
    }
}
impl From<PostRow> for Post {
    fn from(row: PostRow) -> Self {
        Self {
            id: row.id,
            content: row.content,
            author_id: row.author_id,
            channel_id: row.channel_id,
            created_at: row.created_at,
        }
    }
}

pub(in crate::server) struct PostDetailedRow {
    pub(super) post: PostRow,
    pub(super) author: MemberShortRow,
}
impl From<PostDetailedRow> for PostDetailed {
    fn from(row: PostDetailedRow) -> Self {
        Self {
            id: row.post.id,
            content: row.post.content,
            author: MemberShort::from(row.author),
            channel_id: row.post.channel_id,
            created_at: row.post.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NewPost, PostChangeset, PostRow};
    use crate::server::channel::post::error::PostError;
    use shared::models::server::channel::post::{NewPostInput, Post, PostChangesetInput};

    #[test]
    fn creation_normalizes_content_and_binds_ownership() {
        let post = NewPost::try_from((
            4,
            7,
            NewPostInput {
                content: "  hello  ".into(),
            },
        ))
        .unwrap();
        assert_eq!(post.content, "hello");
        assert_eq!(post.author_id, 4);
        assert_eq!(post.channel_id, 7);
    }

    #[test]
    fn changeset_requires_content() {
        assert!(matches!(
            PostChangeset::try_from(PostChangesetInput { content: None }),
            Err(PostError::InvalidChangeset(_))
        ));
    }

    #[test]
    fn row_converts_to_shared_post_contract() {
        let created_at = chrono::NaiveDateTime::default();
        let post = Post::from(PostRow {
            id: 2,
            content: "hello".into(),
            author_id: 4,
            channel_id: 7,
            created_at,
        });
        assert_eq!(post.id, 2);
        assert_eq!(post.author_id, 4);
        assert_eq!(post.channel_id, 7);
    }
}
