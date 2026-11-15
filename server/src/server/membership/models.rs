use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};

use crate::member::models::MemberRow;
use crate::schema::server_membership;
use crate::server::models::ServerRow;
use shared::models::member::MemberId;
use shared::models::server::server::ServerId;

#[derive(Insertable)]
#[diesel(table_name = server_membership)]
pub(in crate::server) struct NewServerMembership {
    member_id: MemberId,
    server_id: ServerId,
}
impl NewServerMembership {
    pub(in crate::server) fn new(member_id: MemberId, server_id: ServerId) -> Self {
        Self {
            member_id,
            server_id,
        }
    }
}

#[derive(Associations, Identifiable, Queryable, Selectable)]
#[diesel(table_name = server_membership)]
#[diesel(primary_key(member_id, server_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(MemberRow, foreign_key = member_id))]
#[diesel(belongs_to(ServerRow, foreign_key = server_id))]
pub(in crate::server) struct ServerMembershipRow {
    member_id: MemberId,
    server_id: ServerId,
    joined_at: NaiveDateTime,
}

#[cfg(test)]
mod tests {
    use super::{NewServerMembership, ServerMembershipRow};

    #[test]
    fn creation_binds_member_and_server() {
        let membership = NewServerMembership::new(4, 9);
        assert_eq!(membership.member_id, 4);
        assert_eq!(membership.server_id, 9);
    }
}
