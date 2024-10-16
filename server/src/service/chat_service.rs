use chrono::NaiveDateTime;
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, OptionalExtension, JoinOnDsl};
use diesel::result::Error;

use crate::db::DbConn;
use crate::models::{
    chat::{Chat, NewChat},
};
use crate::models::member::MemberShort;
use crate::models::message::{ResponseMessage};
use crate::schema::{
    message::dsl as ms_dsl,
    chat::dsl as c_dsl,
    member_chat::dsl as mc_dsl,
    member::dsl as m_dsl
};
use crate::service::CrudOps;

pub struct ChatService;

impl CrudOps<NewChat, Chat> for ChatService {
    fn create(conn: &mut DbConn, new_chat: NewChat) -> Result<Chat, Error> {
        diesel::insert_into(c_dsl::chat)
            .values(&new_chat)
            .get_result::<Chat>(conn)
    }

    fn read(conn: &mut DbConn, id: i32) -> Result<Chat, Error> {
        let chat = c_dsl::chat
            .find(id)
            .first::<Chat>(conn)
            .optional()?;

        match chat {
            Some(chat) => Ok(chat),
            None => Err(Error::NotFound)
        }
    }

    fn update(conn: &mut DbConn, id: i32, entity: NewChat) -> Result<Chat, Error> {
        let updated_fields = (
            c_dsl::name.eq(entity.name().to_owned()),
        );

        diesel::update(c_dsl::chat.find(id))
            .set(updated_fields)
            .get_result::<Chat>(conn)
    }

    fn delete(conn: &mut DbConn, id: i32) -> Result<usize, Error> {
        diesel::delete(c_dsl::chat.find(id)).execute(conn)
    }
}

impl ChatService {
    pub fn get_chat_thread(conn: &mut DbConn, chat_id: i32) -> Result<Vec<ResponseMessage>, Error>{
        let thread = ms_dsl::message
            .filter(ms_dsl::chat_id.eq(chat_id))
            .inner_join(m_dsl::member.on(ms_dsl::sender_id.eq(m_dsl::id)))
            .select((
                ms_dsl::id,
                ms_dsl::content,
                (m_dsl::id, m_dsl::username, m_dsl::avatar),
                ms_dsl::created_at
            ))
            .order_by(ms_dsl::created_at.desc())
            .load::<(i32, String, (i32, String, Option<String>), NaiveDateTime)>(conn)?
            .into_iter()
            .map(|(id, content, (sender_id, username, avatar), created_at)| ResponseMessage {
                id,
                content,
                sender: MemberShort {
                    id: sender_id,
                    username,
                    avatar,
                },
                created_at,
            })
            .collect::<Vec<_>>();

        Ok(thread)
    }

    pub fn get_chat_members(conn: &mut DbConn, chat_id: i32) -> Result<Vec<MemberShort>, Error>{
        let members = mc_dsl::member_chat.filter(mc_dsl::chat_id.eq(chat_id))
            .inner_join(m_dsl::member.on(mc_dsl::member_id.eq(m_dsl::id)))
            .select((m_dsl::id, m_dsl::username, m_dsl::avatar, mc_dsl::joined_at))
            .order_by(mc_dsl::joined_at.asc())
            .load::<(i32, String, Option<String>, NaiveDateTime)>(conn)?
            .into_iter()
            .map(|(id, username, avatar, _)| MemberShort {
                id,
                username,
                avatar
            })
            .collect::<Vec<_>>();

        Ok(members)
    }
}