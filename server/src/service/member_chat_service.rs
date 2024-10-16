use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use diesel::result::{DatabaseErrorKind, Error};

use crate::db::DbConn;
use crate::models::{
    member_chat::{MemberChat, NewMemberChat},
};
use crate::schema::{
    member_chat::dsl as mc_dsl,
};
use crate::service::CrudOps;

pub struct MemberChatService;

impl CrudOps<NewMemberChat, MemberChat> for MemberChatService {
    fn create(conn: &mut DbConn, new_member_chat: NewMemberChat) -> Result<MemberChat, Error> {
        diesel::insert_into(mc_dsl::member_chat)
            .values(&new_member_chat)
            .get_result::<MemberChat>(conn)
    }

    fn read(_conn: &mut DbConn, _id: i32) -> Result<MemberChat, Error> {
        Err(Error::DatabaseError(DatabaseErrorKind::UnableToSendCommand, Box::new(String::new())))
    }

    fn update(_conn: &mut DbConn, _id: i32, _entity: NewMemberChat) -> Result<MemberChat, Error> {
        Err(Error::DatabaseError(DatabaseErrorKind::UnableToSendCommand, Box::new(String::new())))
    }

    fn delete(_conn: &mut DbConn, _id: i32) -> Result<usize, Error> {
        Err(Error::DatabaseError(DatabaseErrorKind::UnableToSendCommand, Box::new(String::new())))
    }
}

impl MemberChatService {
    pub fn join(conn: &mut DbConn, new_member_chat: NewMemberChat) -> Result<MemberChat, Error> {
        Self::create(conn, new_member_chat)
    }
    pub fn is_participant(conn: &mut DbConn, chat_id: i32, member_id: i32) -> bool{
        mc_dsl::member_chat.find((chat_id, member_id))
            .first::<MemberChat>(conn)
            .optional().unwrap().is_some()
    }
    pub fn leave(conn: &mut DbConn, chat_id: i32, member_id: i32) -> Result<usize, Error> {
        diesel::delete(mc_dsl::member_chat.find((chat_id, member_id))).execute(conn)
    }
}