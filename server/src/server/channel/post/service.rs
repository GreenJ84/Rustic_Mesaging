use crate::infra::{Database, DbConn};
use crate::server::channel::repository::ChannelRepository;
use crate::server::membership::repository::ServerMembershipRepository;
use crate::server::repository::ServerRepository;
use shared::models::member::MemberId;
use shared::models::server::channel::channel::ChannelId;
use shared::models::server::channel::post::{
    MultiPostDetailed, NewPostInput, PostChangesetInput, PostDetailed, PostId,
};
use shared::models::server::server::ServerId;

use super::error::PostError;
use super::models::{NewPost, PostChangeset};
use super::repository::PostRepository;

pub(crate) struct PostService {
    database: Database,
}

impl PostService {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }

    fn validate_authorization(
        &self,
        conn: &mut DbConn,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
    ) -> Result<(), PostError> {
        if !ServerRepository::exists(conn, server_id)? {
            return Err(PostError::ServerNotFound);
        }
        if !ServerMembershipRepository::exists(conn, server_id, member_id)? {
            return Err(PostError::MembershipRequired);
        }
        if !ChannelRepository::exists(conn, server_id, channel_id)? {
            return Err(PostError::ChannelNotFound);
        }
        Ok(())
    }

    pub(crate) fn create_post(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        input: NewPostInput,
    ) -> Result<PostDetailed, PostError> {
        let new_post = NewPost::try_from((member_id, channel_id, input))?;
        let mut conn = self.database.get_connection()?;
        conn.transaction::<PostDetailed, PostError, _>(|conn| {
            self.validate_authorization(conn, member_id, server_id, channel_id)?;
            let post = PostRepository::create(conn, new_post)?;
            PostRepository::read_detailed(conn, channel_id, post.id())
                .map(PostDetailed::from)
                .map_err(PostError::from)
        })
    }

    pub(crate) fn get_post_detailed(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<PostDetailed, PostError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<PostDetailed, PostError, _>(|conn| {
            self.validate_authorization(conn, member_id, server_id, channel_id)?;
            PostRepository::read_detailed(conn, channel_id, post_id)
                .map(PostDetailed::from)
                .map_err(PostError::from)
        })
    }

    pub(crate) fn get_post_page(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        page: u32,
    ) -> Result<MultiPostDetailed, PostError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<MultiPostDetailed, PostError, _>(|conn| {
            self.validate_authorization(conn, member_id, server_id, channel_id)?;
            let posts = PostRepository::read_page_detailed(conn, channel_id, page)?
                .into_iter()
                .map(PostDetailed::from)
                .collect();
            Ok(MultiPostDetailed { posts })
        })
    }

    pub(crate) fn update_post(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        post_id: PostId,
        input: PostChangesetInput,
    ) -> Result<PostDetailed, PostError> {
        let changeset = PostChangeset::try_from(input)?;
        let mut conn = self.database.get_connection()?;
        conn.transaction::<PostDetailed, PostError, _>(|conn| {
            self.validate_authorization(conn, member_id, server_id, channel_id)?;
            if member_id != PostRepository::read_author_id(conn, channel_id, post_id)? {
                return Err(PostError::PermissionDenied);
            }
            PostRepository::update(conn, channel_id, post_id, changeset)?;
            PostRepository::read_detailed(conn, channel_id, post_id)
                .map(PostDetailed::from)
                .map_err(PostError::from)
        })
    }

    pub(crate) fn delete_post(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<(), PostError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<(), PostError, _>(|conn| {
            self.validate_authorization(conn, member_id, server_id, channel_id)?;
            if member_id != PostRepository::read_author_id(conn, channel_id, post_id)? {
                return Err(PostError::PermissionDenied);
            }
            PostRepository::delete(conn, channel_id, post_id).map_err(PostError::from)
        })
    }
}
use diesel::Connection;
