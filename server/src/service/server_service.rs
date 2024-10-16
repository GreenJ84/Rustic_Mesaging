use std::collections::HashSet;
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, OptionalExtension, JoinOnDsl, SelectableHelper, PgTextExpressionMethods, TextExpressionMethods};
use diesel::result::Error;
use crate::db::DbConn;
use crate::models::{
    server::{NewServer, Server},
    channel::Channel,
    member::Member
};
use crate::models::member::MemberShort;
use crate::schema::{
    server::dsl as s_dsl,
    member::dsl as m_dsl,
    server_membership::dsl as sm_dsl,
    channel::dsl as c_dsl
};
use crate::service::CrudOps;

pub struct ServerService;

impl CrudOps<NewServer, Server> for ServerService{
    fn create(conn: &mut DbConn, new_server: NewServer) -> Result<Server, Error> {
         diesel::insert_into(s_dsl::server)
            .values(&new_server)
            .get_result::<Server>(conn)
    }

    fn read(conn: &mut DbConn, id: i32) -> Result<Server, Error> {
        let server: Option<Server> = s_dsl::server
            .find(id)
            .first::<Server>(conn)
            .optional()?;

        match server {
            Some(server) => Ok(server),
            None => Err(Error::NotFound)
        }
    }

    fn update(conn: &mut DbConn, id: i32, entity: NewServer) -> Result<Server, Error> {
        let updated_fields = (
            s_dsl::name.eq(entity.name().to_owned()),
            s_dsl::description.eq(entity.description().to_owned()),
        );

        // Update member details by id
         diesel::update(s_dsl::server.find(id))
            .set(updated_fields)
            .get_result::<Server>(conn)
    }

    fn delete(conn: &mut DbConn, id: i32) -> Result<usize, Error> {
        diesel::delete(s_dsl::server.find(id)).execute(conn)
    }
}

impl ServerService{
    pub fn is_owner(conn: &mut DbConn, server_id: i32, member_id: i32) -> bool {
        s_dsl::server.find(server_id)
            .filter(s_dsl::owner_id.eq(member_id))
            .select(s_dsl::id)
            .first::<i32>(conn)
            .optional().unwrap().is_some()
    }
    pub fn search_servers(conn: &mut DbConn, search_term: String) -> Result<Vec<Server>, Error>{
        let mut unique_servers = HashSet::new();

        let mut match_high: Vec<Server> = s_dsl::server
            .filter(s_dsl::name.like(format!("{}%", search_term)))
            .select(Server::as_select())
            .load::<Server>(conn)?
            .into_iter()
            .filter(|server| unique_servers.insert(server.name().to_owned())) // Insert into HashSet
            .collect();

        let match_med: Vec<Server> = s_dsl::server
            .filter(s_dsl::name.ilike(format!("{}%", search_term)))
            .select(Server::as_select())
            .load::<Server>(conn)?
            .into_iter()
            .filter(|server| unique_servers.insert(server.name().to_owned())) // Insert into HashSet
            .collect();

        let match_low: Vec<Server> = s_dsl::server
            .filter(s_dsl::name.ilike(format!("%{}%", search_term)))
            .select(Server::as_select())
            .load::<Server>(conn)?
            .into_iter()
            .filter(|server| unique_servers.insert(server.name().to_owned())) // Insert into HashSet
            .collect();

        match_high.extend(match_med);
        match_high.extend(match_low);
        Ok(match_high)
    }
    pub fn get_all_servers(conn: &mut DbConn) -> Result<Vec<Server>, Error>{
        s_dsl::server
            .select(Server::as_select())
            .order_by(s_dsl::created_at.desc())
            .load::<Server>(conn)
    }
    pub fn get_server_members(conn: &mut DbConn,server_id: i32) -> Result<Vec<Member>, Error>{
        sm_dsl::server_membership
            .filter(sm_dsl::server_id.eq(server_id))
            .inner_join(m_dsl::member.on(sm_dsl::member_id.eq(m_dsl::id)))
            .order_by(sm_dsl::joined_at.asc())
            .select(Member::as_select())
            .load::<Member>(conn)
    }
    pub fn get_server_channels(conn: &mut DbConn, server_id: i32) -> Result<Vec<Channel>, Error>{
       c_dsl::channel
            .filter(c_dsl::server_id.eq(server_id))
            .inner_join(s_dsl::server.on(c_dsl::server_id.eq(s_dsl::id)))
            .order_by(c_dsl::name.asc())
            .select(Channel::as_select())  // Select member ID and username
            .load::<Channel>(conn)
    }
}