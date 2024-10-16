use std::collections::HashSet;
use chrono::NaiveDateTime;
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, OptionalExtension, SelectableHelper, JoinOnDsl, BoolExpressionMethods, PgTextExpressionMethods, TextExpressionMethods};
use diesel::result::Error;

use crate::db::DbConn;
use crate::models::{
    member::{Member, NewMember},
    server::Server,
    chat::Chat
};
use crate::models::friend_request::{MultiFriendRequest, RequestDetailed};
use crate::models::member::MemberShort;
use crate::schema::{
    chat::dsl as c_dsl,
    member_chat::dsl as mc_dsl,
    member::dsl as m_dsl,
    server_membership::dsl as sm_dsl,
    server::dsl as s_dsl,
    friend::dsl as f_dsl,
    friend_request::dsl as fr_dsl
};
use crate::service::CrudOps;

pub struct MemberService;

impl CrudOps<NewMember, Member> for MemberService{
    fn create(conn: &mut DbConn, new_member: NewMember) -> Result<Member, Error> {
        diesel::insert_into(m_dsl::member)
            .values(&new_member)
            .get_result::<Member>(conn)
    }

    fn read(conn: &mut DbConn, id: i32) -> Result<Member, Error> {
        let user = m_dsl::member
            .find(id)
            .first::<Member>(conn)
            .optional()?;

        match user {
            Some(user) => Ok(user),
            None => Err(Error::NotFound)
        }
    }

    fn update(conn: &mut DbConn, id: i32, entity: NewMember) -> Result<Member, Error> {
        let updated_fields = (
            m_dsl::username.eq(entity.username().to_owned()),
            m_dsl::email.eq(entity.email().to_owned()),
        );

        // Update member details by id
        diesel::update(m_dsl::member.filter(m_dsl::id.eq(id)))
            .set(updated_fields)
            .get_result::<Member>(conn)
    }

    fn delete(conn: &mut DbConn, id: i32) -> Result<usize, Error> {
        diesel::delete(m_dsl::member.find(id)).execute(conn)
    }
}

impl MemberService {
    pub fn search_usernames(conn: &mut DbConn, search_term: String) -> Result<Vec<MemberShort>, Error>{
        let mut unique_members = HashSet::new();

        let mut match_high: Vec<MemberShort> = m_dsl::member
            .filter(m_dsl::username.like(format!("{}%", search_term)))
            .select(Member::as_select())
            .load::<Member>(conn)?
            .into_iter()
            .map(|member| member.into_short())
            .filter(|member| unique_members.insert(member.username.clone())) // Insert into HashSet
            .collect();

        let match_med: Vec<MemberShort> = m_dsl::member
            .filter(m_dsl::username.ilike(format!("{}%", search_term)))
            .select(Member::as_select())
            .load::<Member>(conn)?
            .into_iter()
            .map(|member| member.into_short())
            .filter(|member| unique_members.insert(member.username.clone())) // Insert into HashSet
            .collect();

        let match_low: Vec<MemberShort> = m_dsl::member
            .filter(m_dsl::username.ilike(format!("%{}%", search_term)))
            .select(Member::as_select())
            .load::<Member>(conn)?
            .into_iter()
            .map(|member| member.into_short())
            .filter(|member| unique_members.insert(member.username.clone())) // Insert into HashSet
            .collect();

        match_high.extend(match_med);
        match_high.extend(match_low);
        Ok(match_high)
    }
    pub fn find_by_username(conn: &mut DbConn, username: String) -> Result<Member, Error> {
        let member= m_dsl::member
            .filter(m_dsl::username.eq(username))
            .first::<Member>(conn)
            .optional()?;

        match member {
            Some(user) => Ok(user),
            None => Err(Error::NotFound)
        }
    }
    pub fn get_member_servers(conn: &mut DbConn, member_id: i32) -> Result<Vec<Server>, Error>{
        sm_dsl::server_membership
            .filter(sm_dsl::member_id.eq(member_id))
            .inner_join(s_dsl::server.on(sm_dsl::server_id.eq(s_dsl::id)))
            .select(Server::as_select())  // Select server ID and name
            .load::<Server>(conn)
    }

    pub fn get_member_chats(conn: &mut DbConn, member_id: i32) -> Result<Vec<Chat>, Error> {
        mc_dsl::member_chat
            .filter(mc_dsl::member_id.eq(member_id))
            .inner_join(c_dsl::chat.on(mc_dsl::chat_id.eq(c_dsl::id)))
            .select(Chat::as_select())
            .load::<Chat>(conn)
    }

    pub fn get_member_friends(conn: &mut DbConn, member_id: i32) -> Result<Vec<(NaiveDateTime, Member)>, Error>{
        f_dsl::friend
            .filter(
                f_dsl::member_id.eq(member_id)
                    .or(f_dsl::friend_id.eq(member_id))  // Find all friend relationships
            )
            .inner_join(m_dsl::member.on(
                // Join with the member table on the friend’s ID
                diesel::dsl::sql::<diesel::sql_types::Bool>("CASE WHEN friend.member_id = $1 THEN friend.friend_id = member.id ELSE friend.member_id = member.id END")
            ))
            .select((f_dsl::created_at, Member::as_select()))  // Select server ID and name
            .load::<(NaiveDateTime, Member)>(conn)
    }

    pub fn get_member_requests(conn: &mut DbConn, member_id: i32) -> Result<MultiFriendRequest, Error>{
        let incoming: Vec<RequestDetailed> = fr_dsl::friend_request
            .filter(fr_dsl::receiver_id.eq(member_id))
            .inner_join(m_dsl::member.on(m_dsl::id.eq(fr_dsl::sender_id)))
            .select((fr_dsl::created_at, fr_dsl::note, Member::as_select()))
            .load::<(NaiveDateTime, Option<String>, Member)>(conn)?
            .into_iter()
            .map(|entry| {
                RequestDetailed {member: entry.2, note: entry.1, created_at: entry.0}
            })
            .collect::<Vec<RequestDetailed>>();

        let outgoing: Vec<RequestDetailed> = fr_dsl::friend_request
            .filter(fr_dsl::sender_id.eq(member_id))
            .inner_join(m_dsl::member.on(m_dsl::id.eq(fr_dsl::receiver_id)))
            .select((fr_dsl::created_at, fr_dsl::note, Member::as_select()))
            .load::<(NaiveDateTime, Option<String>, Member)>(conn)?
            .into_iter()
            .map(|entry| {
                RequestDetailed {member: entry.2, note: entry.1, created_at: entry.0}
            })
            .collect::<Vec<RequestDetailed>>();;

        Ok(MultiFriendRequest { incoming, outgoing })
    }

    pub fn update_password(conn: &mut DbConn, id: i32, new_pass: String) -> Result<Member, Error> {
        let updated_fields = (
            m_dsl::password.eq(new_pass.to_owned()),
        );

        // Update member details by id
        diesel::update(m_dsl::member.filter(m_dsl::id.eq(id)))
            .set(updated_fields)
            .get_result::<Member>(conn)
    }
}

