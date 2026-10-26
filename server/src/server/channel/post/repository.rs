use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::database::{DbConn, DbError};
use crate::member::models::{MemberId, MemberShort};
use crate::schema::{channel::dsl as c_dsl, member::dsl as m_dsl, post::dsl as p_dsl};
use crate::server::channel::models::ChannelId;
use crate::server::channel::post::models::PostChangeset;
use crate::utils::Paginator;

use super::models::{NewPost, Post, PostDetail, PostId};

const POST_PAGINATOR: Paginator = Paginator::new(None);

pub(super) struct PostRepository;
impl PostRepository {
    // Create
    pub(super) fn create(conn: &mut DbConn, new_post: NewPost) -> Result<Post, DbError> {
        diesel::insert_into(p_dsl::post)
            .values(&new_post)
            .returning(Post::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(super) fn read(
        conn: &mut DbConn,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<Post, DbError> {
        p_dsl::post
            .find(post_id)
            .filter(p_dsl::channel_id.eq(channel_id))
            .select(Post::as_select())
            .first(conn)
            .map_err(DbError::from)
    }
    pub(super) fn read_author_id(
        conn: &mut DbConn,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<MemberId, DbError> {
        p_dsl::post
            .find(post_id)
            .filter(p_dsl::channel_id.eq(channel_id))
            .select(p_dsl::author_id)
            .first::<MemberId>(conn)
            .map_err(DbError::from)
    }
    pub(super) fn read_detail(
        conn: &mut DbConn,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<PostDetail, DbError> {
        p_dsl::post
            .find(post_id)
            .filter(p_dsl::channel_id.eq(channel_id))
            .inner_join(m_dsl::member.on(p_dsl::author_id.eq(m_dsl::id)))
            .select((Post::as_select(), MemberShort::as_select()))
            .first::<(Post, MemberShort)>(conn)
            .map(|(post, member)| PostDetail::from((post, member)))
            .map_err(DbError::from)
    }
    pub(super) fn read_page(
        conn: &mut DbConn,
        channel_id: ChannelId,
        page: u32,
    ) -> Result<Vec<PostDetail>, DbError> {
        let page_size = POST_PAGINATOR.page_size();
        let page_offset = POST_PAGINATOR.offset(page);

        p_dsl::post
            .filter(p_dsl::channel_id.eq(channel_id))
            .inner_join(m_dsl::member.on(p_dsl::author_id.eq(m_dsl::id)))
            .order_by((p_dsl::created_at.desc(), p_dsl::id.desc()))
            .limit(page_size)
            .offset(page_offset)
            .select((Post::as_select(), MemberShort::as_select()))
            .load::<(Post, MemberShort)>(conn)
            .map(|results| {
                results
                    .into_iter()
                    .map(|(post, member)| PostDetail::from((post, member)))
                    .collect()
            })
            .map_err(DbError::from)
    }

    // Update
    pub(super) fn update(
        conn: &mut DbConn,
        channel_id: ChannelId,
        post_id: PostId,
        changeset: PostChangeset,
    ) -> Result<Post, DbError> {
        diesel::update(
            p_dsl::post
                .find(post_id)
                .filter(p_dsl::channel_id.eq(channel_id)),
        )
        .set(changeset)
        .returning(Post::as_returning())
        .get_result(conn)
        .map_err(DbError::from)
    }

    // Delete
    pub(super) fn delete(
        conn: &mut DbConn,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<(), DbError> {
        let size = diesel::delete(
            p_dsl::post
                .find(post_id)
                .filter(p_dsl::channel_id.eq(channel_id)),
        )
        .execute(conn)
        .map_err(DbError::from)?;

        if size == 0 {
            Err(DbError::NotFound)
        } else {
            Ok(())
        }
    }
}
