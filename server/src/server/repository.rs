use diesel::dsl::case_when;
use diesel::sql_types::Integer;
use diesel::{
    EscapeExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper, TextExpressionMethods,
};

use crate::database::{Database, DbConn, DbError};
use crate::member::models::MemberId;
use crate::schema::server::dsl as s_dsl;

use super::models::{NewServer, Server, ServerChangeset, ServerId};

pub(crate) struct ServerRepository;
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
    pub(super) fn name_taken(conn: &mut DbConn, name: &str) -> Result<bool, DbError> {
        diesel::dsl::select(diesel::dsl::exists(
            s_dsl::server.filter(s_dsl::name.eq(name)),
        ))
        .get_result::<bool>(conn)
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

    pub(super) fn read_page(conn: &mut DbConn, page: u32) -> Result<Vec<Server>, DbError> {
        let offset = Database::page_offset(page);
        s_dsl::server
            .select(Server::as_select())
            .order_by((s_dsl::created_at.desc(), s_dsl::id.desc()))
            .limit(Database::PAGE_SIZE)
            .offset(offset)
            .load(conn)
            .map_err(DbError::from)
    }

    pub(super) fn search(conn: &mut DbConn, search_term: String) -> Result<Vec<Server>, DbError> {
        let escaped = Database::escape_search_pattern(&search_term);

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

    // pub fn get_server_channels(conn: &mut DbConn, server_id: i32) -> Result<Vec<Channel>, DbError> {
    //     c_dsl::channel
    //         .filter(c_dsl::server_id.eq(server_id))
    //         .inner_join(s_dsl::server.on(c_dsl::server_id.eq(s_dsl::id)))
    //         .order_by(c_dsl::name.asc())
    //         .select(Channel::as_select()) // Select member ID and username
    //         .load::<Channel>(conn)
    // }
}

