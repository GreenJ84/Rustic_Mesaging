use chrono::NaiveDateTime;
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, OptionalExtension, JoinOnDsl};
use diesel::result::Error;
use crate::db::DbConn;
use crate::models::channel::{NewChannel, Channel};
use crate::models::member::MemberShort;
use crate::models::post::{MultiPost, PostDetail};
use crate::schema::{
    post::dsl as p_dsl,
    channel::dsl as c_dsl,
    member::dsl as m_dsl
};
use crate::service::CrudOps;
use crate::service::server_membership_service::MembershipService;

pub struct ChannelService;


impl CrudOps<NewChannel, Channel> for ChannelService{
    fn create(conn: &mut DbConn, new_channel: NewChannel) -> Result<Channel, Error> {
        diesel::insert_into(c_dsl::channel)
            .values(&new_channel)
            .get_result::<Channel>(conn)
    }

    fn read(conn: &mut DbConn, id: i32) -> Result<Channel, Error> {
        let channel: Option<Channel> = c_dsl::channel
            .find(id)
            .first::<Channel>(conn)
            .optional()
            .map_err(|e| e)?;

        match channel {
            Some(user) => Ok(user),
            None => Err(Error::NotFound)
        }
    }

    fn update(conn: &mut DbConn, id: i32, entity: NewChannel) -> Result<Channel, Error> {
        let updated_fields = (
            c_dsl::name.eq(entity.name().to_owned()),
        );

        diesel::update(c_dsl::channel.find(id))
            .set(updated_fields)
            .get_result::<Channel>(conn)
    }

    fn delete(conn: &mut DbConn, id: i32) -> Result<usize, Error> {
        diesel::delete(c_dsl::channel.find(id)).execute(conn)
    }
}
impl ChannelService {
    pub fn has_authorization(
        conn: &mut DbConn,
        channel_id: i32,
        member_id: i32
    ) -> bool {
        let server_id = c_dsl::channel
            .find(channel_id)
            .select(c_dsl::server_id)
            .first::<i32>(conn)
            .optional().unwrap().unwrap();

        MembershipService::has_membership(conn, server_id, member_id)
    }
    pub fn get_all_channels(conn: &mut DbConn) -> Result<Vec<Channel>, Error>{
        c_dsl::channel
            .order_by(c_dsl::name.asc())
            .load::<Channel>(conn)
    }
    pub fn get_channel_posts(conn: &mut DbConn, channel_id: i32) -> Result<MultiPost, Error>{
        let posts = p_dsl::post
            .filter(p_dsl::channel_id.eq(channel_id))
            .inner_join(m_dsl::member.on(p_dsl::author_id.eq(m_dsl::id)))
            .select((
                p_dsl::id,
                p_dsl::content,
                (m_dsl::id, m_dsl::username, m_dsl::avatar),
                p_dsl::channel_id,
                p_dsl::created_at
            ))
            .order_by(p_dsl::created_at.desc())// Select member ID and username
            .load::<(i32, String, (i32, String, Option<String>), i32, NaiveDateTime)>(conn)?
            .into_iter()
            .map(|(id, content, (member_id, username, avatar), channel_id, timestamp)|
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
            )
            .collect::<Vec<PostDetail>>();

        Ok(MultiPost { posts })
    }
}