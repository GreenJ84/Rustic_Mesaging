use diesel::dsl::case_when;
use diesel::sql_types::Integer;
use diesel::{
    EscapeExpressionMethods, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper, TextExpressionMethods,
};

use crate::database::{Database, DbConn, DbError};
use crate::member::models::MemberId;
use crate::schema::server::dsl as s_dsl;
use crate::utils::{Paginator, SearchValidation};

use super::models::{NewServer, Server, ServerChangeset, ServerId};

const SERVER_PAGINATOR: Paginator = Paginator::new(None);

pub(crate) struct ServerRepository;
impl ServerRepository {
    // Existence
    pub(super) fn exists(conn: &mut DbConn, id: ServerId) -> Result<bool, DbError> {
        diesel::dsl::select(diesel::dsl::exists(s_dsl::server.find(id)))
            .get_result::<bool>(conn)
            .map_err(DbError::from)
    }

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
        let page_offset = SERVER_PAGINATOR.offset(page);
        let page_size = SERVER_PAGINATOR.page_size();
        s_dsl::server
            .select(Server::as_select())
            .order_by((s_dsl::created_at.desc(), s_dsl::id.desc()))
            .limit(page_size)
            .offset(page_offset)
            .load(conn)
            .map_err(DbError::from)
    }
    pub(super) fn search(
        conn: &mut DbConn,
        search: SearchValidation,
    ) -> Result<Vec<Server>, DbError> {
        let prefix = format!("{}%", search.escaped);
        let contains = format!("%{}%", search.escaped);

        let relevance = case_when::<_, _, Integer>(s_dsl::name.eq(search.normalized), 0)
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
}
