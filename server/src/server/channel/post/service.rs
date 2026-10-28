use crate::database::Database;
use crate::member::{models::MemberId, repository::MemberRepository};
use crate::server::channel::{models::ChannelId, repository::ChannelRepository};
use crate::server::membership::repository::ServerMembershipRepository;
use crate::server::{models::ServerId, repository::ServerRepository};

use super::error::PostError;
use super::models::*;
use super::repository::PostRepository;

pub struct PostService {
    database: Database,
}
impl PostService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    fn validated_and_authorized(
        &self,
        conn: &mut crate::database::DbConn,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
    ) -> Result<(), PostError> {
        if !ServerRepository::exists(conn, server_id)? {
            return Err(PostError::ServerNotFound);
        } else if !ServerMembershipRepository::exists(conn, server_id, member_id)? {
            return Err(PostError::MembershipRequired);
        } else if !ChannelRepository::exists(conn, server_id, channel_id)? {
            return Err(PostError::ChannelNotFound);
        }
        Ok(())
    }

    // Create
    pub fn create_post(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        content: NewPostInput,
    ) -> Result<Post, PostError> {
        let new_post = NewPost::try_from((member_id, channel_id, content))?;

        let mut conn = self.database.get_connection()?;

        self.validated_and_authorized(&mut conn, member_id, server_id, channel_id)?;

        PostRepository::create(&mut conn, new_post).map_err(PostError::from)
    }

    // Read
    pub fn get_post(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<Post, PostError> {
        let mut conn = self.database.get_connection()?;

        self.validated_and_authorized(&mut conn, member_id, server_id, channel_id)?;

        PostRepository::read(&mut conn, channel_id, post_id).map_err(PostError::from)
    }

    pub fn get_post_detail(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<PostDetail, PostError> {
        let mut conn = self.database.get_connection()?;

        self.validated_and_authorized(&mut conn, member_id, server_id, channel_id)?;

        PostRepository::read_detail(&mut conn, channel_id, post_id).map_err(PostError::from)
    }

    pub fn get_post_page(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        page: u32,
    ) -> Result<MultiPost, PostError> {
        let mut conn = self.database.get_connection()?;

        self.validated_and_authorized(&mut conn, member_id, server_id, channel_id)?;

        PostRepository::read_page(&mut conn, channel_id, page).map_err(PostError::from)
    }

    // Update
    pub fn update_post(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        post_id: PostId,
        changeset: PostChangesetInput,
    ) -> Result<Post, PostError> {
        let changeset = PostChangeset::try_from(changeset)?;
        let mut conn = self.database.get_connection()?;

        self.validated_and_authorized(&mut conn, member_id, server_id, channel_id)?;
        if member_id != PostRepository::read_author_id(&mut conn, channel_id, post_id)? {
            return Err(PostError::PermissionDenied);
        }

        PostRepository::update(&mut conn, channel_id, post_id, changeset).map_err(PostError::from)
    }

    // Delete
    pub fn delete_post(
        &self,
        member_id: MemberId,
        server_id: ServerId,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<(), PostError> {
        let mut conn = self.database.get_connection()?;

        self.validated_and_authorized(&mut conn, member_id, server_id, channel_id)?;
        if member_id != PostRepository::read_author_id(&mut conn, channel_id, post_id)? {
            return Err(PostError::PermissionDenied);
        }

        PostRepository::delete(&mut conn, channel_id, post_id).map_err(PostError::from)
    }
}
