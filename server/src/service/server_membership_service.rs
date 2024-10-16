use diesel::result::{DatabaseErrorKind, Error};
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use crate::db::DbConn;
use crate::models::server_membership::{NewServerMembership, ServerMembership};
use crate::schema::server_membership::dsl;
use crate::service::CrudOps;

pub struct MembershipService;

impl MembershipService {
    pub fn join(conn: &mut DbConn,  new_membership: NewServerMembership) -> Result<ServerMembership, Error> {
        diesel::insert_into(dsl::server_membership)
            .values(&new_membership)
            .get_result::<ServerMembership>(conn)
    }
    pub fn has_membership(
        conn: &mut DbConn, server_id: i32, member_id: i32
    ) -> bool {
        dsl::server_membership.find((member_id, server_id))
            .first::<ServerMembership>(conn)
            .optional().unwrap().is_some()
    }
    pub fn leave(conn: &mut DbConn, server_id: i32, member_id: i32) -> Result<usize, Error> {
        diesel::delete(dsl::server_membership.find((member_id, server_id))).execute(conn)
    }
}