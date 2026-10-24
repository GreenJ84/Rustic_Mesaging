use diesel::dsl::case_when;
use diesel::sql_types::Integer;
use diesel::{
    EscapeExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper, TextExpressionMethods,
};

use crate::database::{DbConn, DbError};
use crate::member::models::MemberId;
use crate::schema::server::dsl as s_dsl;

use super::models::{NewServer, Server, ServerChangeset, ServerId};

pub(super) struct ServerRepository;
impl ServerRepository {
    // Create
    pub(super) fn create(conn: &mut DbConn, new_server: NewServer) -> Result<Server, DbError> {
        diesel::insert_into(s_dsl::server)
            .values(new_server)
            .get_result::<Server>(conn)
            .map_err(DbError::from)
    }

    // Read
    pub(super) fn read(conn: &mut DbConn, id: ServerId) -> Result<Server, DbError> {
        s_dsl::server
            .find(id)
            .first::<Server>(conn)
            .map_err(DbError::from)
    }
    pub(super) fn is_owner(
        conn: &mut DbConn,
        server_id: ServerId,
        member_id: MemberId,
    ) -> Result<bool, DbError> {
        diesel::dsl::select(diesel::dsl::exists(
            s_dsl::server
                .find(server_id)
                .filter(s_dsl::owner_id.eq(member_id)),
        ))
        .get_result::<bool>(conn)
        .map_err(DbError::from)
    }

    const PAGE_SIZE: i64 = 25;
    pub(super) fn read_page(conn: &mut DbConn, page: u32) -> Result<Vec<Server>, DbError> {
        let offset = Self::page_offset(page);
        s_dsl::server
            .select(Server::as_select())
            .order_by((s_dsl::created_at.desc(), s_dsl::id.desc()))
            .limit(Self::PAGE_SIZE)
            .offset(offset)
            .load(conn)
            .map_err(DbError::from)
    }
    fn page_offset(page: u32) -> i64 {
        i64::from(page) * Self::PAGE_SIZE
    }
    pub(super) fn search(conn: &mut DbConn, search_term: String) -> Result<Vec<Server>, DbError> {
        let escaped = Self::escape_like_pattern(&search_term);

        let prefix = format!("{escaped}%");
        let contains = format!("%{escaped}%");

        let relevance = case_when::<_, _, Integer>(s_dsl::name.eq(search_term), 0)
            .when(s_dsl::name.like(&prefix).escape('\\'), 1)
            .when(s_dsl::name.ilike(&prefix).escape('\\'), 2)
            .otherwise(3);

        s_dsl::server
            .filter(s_dsl::name.ilike(contains).escape('\\'))
            .order_by((relevance.asc(), s_dsl::name.asc(), s_dsl::id.asc()))
            .limit(25)
            .select(Server::as_select())
            .load(conn)
            .map_err(DbError::from)
    }
    fn escape_like_pattern(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len());
        for character in value.chars() {
            if matches!(character, '\\' | '%' | '_') {
                escaped.push('\\');
            }
            escaped.push(character);
        }
        escaped
    }

    // Update
    pub(super) fn update(
        conn: &mut DbConn,
        id: ServerId,
        changeset: ServerChangeset,
    ) -> Result<Server, DbError> {
        diesel::update(s_dsl::server.find(id))
            .set(changeset)
            .get_result::<Server>(conn)
            .map_err(DbError::from)
    }

    // Delete
    pub(super) fn delete(conn: &mut DbConn, id: ServerId) -> Result<(), DbError> {
        let size = diesel::delete(s_dsl::server.find(id))
            .execute(conn)
            .map_err(DbError::from)?;

        if size == 0 {
            Err(DbError::NotFound)
        } else {
            Ok(())
        }
    }

    // pub fn get_member_servers(conn: &mut DbConn, member_id: i32) -> Result<Vec<Server>, DbError> {
    //     sm_dsl::server_membership
    //         .filter(sm_dsl::member_id.eq(member_id))
    //         .inner_join(s_dsl::server.on(sm_dsl::server_id.eq(s_dsl::id)))
    //         .select(Server::as_select()) // Select server ID and name
    //         .load::<Server>(conn)
    //         .map_err(DbError::from)
    // }
    // pub fn get_all_member_servers(&self, member_id: MemberId) -> Result<MultiServer, MemberDbError> {
    //     let mut conn = self.database.get_connection()?;
    //     Ok(MultiServer {
    //         servers: MemberRepository::get_member_servers(&mut conn, member_id)?,
    //     })
    // }
    // pub fn get_server_members(conn: &mut DbConn, server_id: i32) -> Result<Vec<Member>, DbError> {
    //     sm_dsl::server_membership
    //         .filter(sm_dsl::server_id.eq(server_id))
    //         .inner_join(m_dsl::member.on(sm_dsl::member_id.eq(m_dsl::id)))
    //         .order_by(sm_dsl::joined_at.asc())
    //         .select(Member::as_select())
    //         .load::<Member>(conn)
    // }
    // pub fn get_server_channels(conn: &mut DbConn, server_id: i32) -> Result<Vec<Channel>, DbError> {
    //     c_dsl::channel
    //         .filter(c_dsl::server_id.eq(server_id))
    //         .inner_join(s_dsl::server.on(c_dsl::server_id.eq(s_dsl::id)))
    //         .order_by(c_dsl::name.asc())
    //         .select(Channel::as_select()) // Select member ID and username
    //         .load::<Channel>(conn)
    // }
}

#[cfg(test)]
mod tests {
    use super::ServerRepository;

    #[test]
    fn escapes_like_wildcards_and_escape_characters() {
        assert_eq!(
            ServerRepository::escape_like_pattern(r"50%_off\today"),
            r"50\%\_off\\today"
        );
    }

    #[test]
    fn leaves_plain_search_terms_unchanged() {
        assert_eq!(
            ServerRepository::escape_like_pattern("rust server"),
            "rust server"
        );
    }

    #[test]
    fn calculates_zero_based_page_offsets() {
        assert_eq!(ServerRepository::page_offset(0), 0);
        assert_eq!(ServerRepository::page_offset(1), 25);
        assert_eq!(ServerRepository::page_offset(3), 75);
        assert_eq!(
            ServerRepository::page_offset(u32::MAX),
            i64::from(u32::MAX) * 25
        );
    }
}
