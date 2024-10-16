use chrono::NaiveDateTime;
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, OptionalExtension, JoinOnDsl};
use diesel::result::Error;
use crate::db::DbConn;
use crate::models::member::MemberShort;
use crate::models::post::{NewPost, Post, PostDetail};
use crate::schema::{
    channel::dsl as c_dsl,
    post::dsl as p_dsl,
    member::dsl as m_dsl
};
use crate::service::CrudOps;
use crate::service::server_membership_service::MembershipService;

pub struct PostService;


impl CrudOps<NewPost, Post> for PostService{
    fn create(conn: &mut DbConn, new_post: NewPost) -> Result<Post, Error> {
         diesel::insert_into(p_dsl::post)
            .values(&new_post)
            .get_result::<Post>(conn)
    }

    fn read(conn: &mut DbConn, id: i32) -> Result<Post, Error> {
        let post = p_dsl::post
            .find(id)
            .first::<Post>(conn)
            .optional()?;

        match post {
            Some(post) => Ok(post),
            None => Err(Error::NotFound)
        }
    }

    fn update(conn: &mut DbConn, id: i32, entity: NewPost) -> Result<Post, Error> {
        let updated_fields = (
            p_dsl::content.eq(entity.content().to_owned()),
        );

        // Update member details by id
         diesel::update(p_dsl::post.find(id))
            .set(updated_fields)
            .get_result::<Post>(conn)
    }

    fn delete(conn: &mut DbConn, id: i32) -> Result<usize, Error> {
        diesel::delete(p_dsl::post.find(id)).execute(conn)
    }
}
impl PostService {
    pub fn has_authorization(
        conn: &mut DbConn,
        post_id: i32,
        member_id: i32
    ) -> bool {
        let server_id = p_dsl::post
            .find(post_id)
            .inner_join(c_dsl::channel.on(p_dsl::channel_id.eq(c_dsl::id)))
            .select(c_dsl::server_id)
            .first::<i32>(conn)
            .optional().unwrap().unwrap();

        MembershipService::has_membership(conn, server_id, member_id)
    }
    pub fn read_detail(conn: &mut DbConn, id: i32) -> Result<PostDetail, Error> {
        let post = p_dsl::post
            .find(id)
            .inner_join(m_dsl::member.on(p_dsl::author_id.eq(m_dsl::id)))
            .select((
                p_dsl::id,
                p_dsl::content,
                (m_dsl::id, m_dsl::username, m_dsl::avatar),
                p_dsl::channel_id,
                p_dsl::created_at
            ))
            .first::<(i32, String, (i32, String, Option<String>), i32, NaiveDateTime)>(conn)
            .map(|(id, content, (member_id, username, avatar), channel_id, timestamp)| {
                PostDetail::new(
                    id,
                    content,
                    MemberShort {
                        id: member_id,
                        username,
                        avatar
                    },
                    channel_id,
                    timestamp
                )
            })
            .optional()?;

        match post {
            Some(post) => Ok(post),
            None => Err(Error::NotFound)
        }
    }
}