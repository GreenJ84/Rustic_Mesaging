use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::database::{DbConn, DbError};
use crate::member::models::MemberShort;
use crate::schema::{channel::dsl as c_dsl, member::dsl as m_dsl, post::dsl as p_dsl};
use crate::server::channel::post::models::PostChangeset;

use super::models::{NewPost, Post, PostDetail, PostId};

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
    pub(super) fn read(conn: &mut DbConn, post_id: PostId) -> Result<Post, DbError> {
        p_dsl::post
            .find(post_id)
            .select(Post::as_select())
            .first(conn)
            .map_err(DbError::from)
    }
    pub(super) fn read_detail(conn: &mut DbConn, post_id: PostId) -> Result<PostDetail, DbError> {
        p_dsl::post
            .find(post_id)
            .inner_join(m_dsl::member.on(p_dsl::author_id.eq(m_dsl::id)))
            .select((Post::as_select(), MemberShort::as_select()))
            .first::<(Post, MemberShort)>(conn)
            .map(|(post, member)| PostDetail::from((post, member)))
            .map_err(DbError::from)
    }

    // Update
    pub(super) fn update(
        conn: &mut DbConn,
        post_id: PostId,
        changeset: PostChangeset,
    ) -> Result<Post, DbError> {
        diesel::update(p_dsl::post.find(post_id))
            .set(changeset)
            .returning(Post::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Delete
    pub(super) fn delete(conn: &mut DbConn, post_id: PostId) -> Result<(), DbError> {
        let size = diesel::delete(p_dsl::post.find(post_id))
            .execute(conn)
            .map_err(DbError::from)?;

        if size == 0 {
            Err(DbError::NotFound)
        } else {
            Ok(())
        }
    }
}
