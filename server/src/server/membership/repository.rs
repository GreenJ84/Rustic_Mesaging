use diesel::{
    ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};

use crate::infra::{DbConn, DbError};
use crate::member::models::{MemberId, MemberShort};
use crate::schema::{member::dsl as m_dsl, server::dsl as s_dsl, server_membership::dsl as sm_dsl};
use crate::server::models::{ServerId, ServerPreview};
use crate::utils::Paginator;

use super::models::{NewServerMembership, ServerMembership};

const MEMBERSHIP_PAGINATOR: Paginator = Paginator::new(None);
pub(in crate::server) struct ServerMembershipRepository;
impl ServerMembershipRepository {
    // Create
    pub(in crate::server) fn create(
        conn: &mut DbConn,
        new_membership: NewServerMembership,
    ) -> Result<(), DbError> {
        let size = diesel::insert_into(sm_dsl::server_membership)
            .values(&new_membership)
            .execute(conn)
            .map_err(DbError::from)?;

        Ok(())
    }

    // Read
    pub(in crate::server) fn exists(
        conn: &mut DbConn,
        server_id: ServerId,
        member_id: MemberId,
    ) -> Result<bool, DbError> {
        diesel::dsl::select(diesel::dsl::exists(
            sm_dsl::server_membership.find((member_id, server_id)),
        ))
        .get_result::<bool>(conn)
        .map_err(DbError::from)
    }
    pub(in crate::server) fn get_member_servers(
        conn: &mut DbConn,
        member_id: MemberId,
        page: u32,
    ) -> Result<Vec<ServerPreview>, DbError> {
        let page_offset = MEMBERSHIP_PAGINATOR.offset(page);
        let page_size = MEMBERSHIP_PAGINATOR.page_size();
        sm_dsl::server_membership
            .filter(sm_dsl::member_id.eq(member_id))
            .inner_join(s_dsl::server.on(sm_dsl::server_id.eq(s_dsl::id)))
            .order_by((sm_dsl::joined_at.desc(), s_dsl::id.desc()))
            .limit(page_size)
            .offset(page_offset)
            .select(ServerPreview::as_select())
            .load(conn)
            .map_err(DbError::from)
    }
    pub(in crate::server) fn get_server_members(
        conn: &mut DbConn,
        server_id: ServerId,
        page: u32,
    ) -> Result<Vec<MemberShort>, DbError> {
        let page_offset = MEMBERSHIP_PAGINATOR.offset(page);
        let page_size = MEMBERSHIP_PAGINATOR.page_size();
        sm_dsl::server_membership
            .filter(sm_dsl::server_id.eq(server_id))
            .inner_join(m_dsl::member.on(sm_dsl::member_id.eq(m_dsl::id)))
            .order_by((m_dsl::username.asc(), m_dsl::id.desc()))
            .limit(page_size)
            .offset(page_offset)
            .select(MemberShort::as_select())
            .load(conn)
            .map_err(DbError::from)
    }

    // Delete
    pub(in crate::server) fn delete(
        conn: &mut DbConn,
        server_id: ServerId,
        member_id: MemberId,
    ) -> Result<(), DbError> {
        let size = diesel::delete(sm_dsl::server_membership.find((member_id, server_id)))
            .execute(conn)
            .map_err(DbError::from)?;
        if size == 0 {
            Err(DbError::NotFound)
        } else {
            Ok(())
        }
    }
}
