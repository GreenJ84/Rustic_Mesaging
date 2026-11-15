use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::infra::{DbConn, DbError};
use crate::member::models::MemberShortRow;
use crate::schema::{channel::dsl as c_dsl, member::dsl as m_dsl, post::dsl as p_dsl};
use crate::server::channel::post::models::PostChangeset;
use crate::utils::Paginator;
use shared::models::member::MemberId;
use shared::models::server::channel::channel::ChannelId;
use shared::models::server::channel::post::PostId;

use super::models::{NewPost, PostDetailedRow, PostRow};

const POST_PAGINATOR: Paginator = Paginator::new(None);

pub(in crate::server) struct PostRepository;
impl PostRepository {
    // Create
    pub(in crate::server) fn create(
        conn: &mut DbConn,
        new_post: NewPost,
    ) -> Result<PostRow, DbError> {
        diesel::insert_into(p_dsl::post)
            .values(&new_post)
            .returning(PostRow::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(in crate::server) fn read(
        conn: &mut DbConn,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<PostRow, DbError> {
        p_dsl::post
            .find(post_id)
            .filter(p_dsl::channel_id.eq(channel_id))
            .select(PostRow::as_select())
            .first(conn)
            .map_err(DbError::from)
    }
    pub(in crate::server) fn read_author_id(
        conn: &mut DbConn,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<MemberId, DbError> {
        p_dsl::post
            .find(post_id)
            .filter(p_dsl::channel_id.eq(channel_id))
            .select(p_dsl::author_id)
            .first(conn)
            .map_err(DbError::from)
    }
    pub(in crate::server) fn read_detailed(
        conn: &mut DbConn,
        channel_id: ChannelId,
        post_id: PostId,
    ) -> Result<PostDetailedRow, DbError> {
        p_dsl::post
            .find(post_id)
            .filter(p_dsl::channel_id.eq(channel_id))
            .inner_join(m_dsl::member.on(p_dsl::author_id.eq(m_dsl::id)))
            .select((PostRow::as_select(), MemberShortRow::as_select()))
            .first(conn)
            .map(|(post, author)| PostDetailedRow { post, author })
            .map_err(DbError::from)
    }
    pub(in crate::server) fn read_page_detailed(
        conn: &mut DbConn,
        channel_id: ChannelId,
        page: u32,
    ) -> Result<Vec<PostDetailedRow>, DbError> {
        let page_size = POST_PAGINATOR.page_size();
        let page_offset = POST_PAGINATOR.offset(page);

        p_dsl::post
            .filter(p_dsl::channel_id.eq(channel_id))
            .inner_join(m_dsl::member.on(p_dsl::author_id.eq(m_dsl::id)))
            .order_by((p_dsl::created_at.desc(), p_dsl::id.desc()))
            .limit(page_size)
            .offset(page_offset)
            .select((PostRow::as_select(), MemberShortRow::as_select()))
            .load(conn)
            .map(|results| {
                results
                    .into_iter()
                    .map(|(post, author)| PostDetailedRow { post, author })
                    .collect()
            })
            .map_err(DbError::from)
    }

    // Update
    pub(in crate::server) fn update(
        conn: &mut DbConn,
        channel_id: ChannelId,
        post_id: PostId,
        changeset: PostChangeset,
    ) -> Result<PostRow, DbError> {
        diesel::update(
            p_dsl::post
                .find(post_id)
                .filter(p_dsl::channel_id.eq(channel_id)),
        )
        .set(changeset)
        .returning(PostRow::as_returning())
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
