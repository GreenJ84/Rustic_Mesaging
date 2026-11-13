use diesel::dsl::case_when;
use diesel::prelude::*;
use diesel::sql_types::Integer;

use super::models::{
    MemberChangeset, MemberDetailed, MemberDetailedRow, MemberId, MemberRow, MemberShort,
    MemberShortRow, NewMember,
};
use crate::infra::{DbConn, DbError};
use crate::schema::member::dsl as m_dsl;
use crate::utils::SearchValidation;

enum MemberIdentifier<'a> {
    Id(MemberId),
    Username(&'a str),
}

/// Stateless persistence operations for members.
pub(crate) struct MemberRepository;
impl MemberRepository {
    // Existence
    pub(crate) fn identity_exists(
        conn: &mut DbConn,
        username: &str,
        email: &str,
    ) -> Result<Option<String>, DbError> {
        m_dsl::member
            .filter(
                m_dsl::username.eq(username).or(m_dsl::email.eq(email)), // Check if either username or email matches
            )
            .select(m_dsl::username)
            .first::<String>(conn)
            .optional()
            .map_err(DbError::from)
    }
    pub(crate) fn identity_exists_excluding(
        conn: &mut DbConn,
        member_id: MemberId,
        username: &str,
        email: &str,
    ) -> Result<Option<String>, DbError> {
        m_dsl::member
            .filter(
                m_dsl::id
                    .ne(member_id)
                    .and(m_dsl::username.eq(username).or(m_dsl::email.eq(email))), // Check if either username or email matches, excluding the current member
            )
            .select(m_dsl::username)
            .first::<String>(conn)
            .optional()
            .map_err(DbError::from)
    }
    pub(crate) fn exists(conn: &mut DbConn, id: MemberId) -> Result<bool, DbError> {
        diesel::select(diesel::dsl::exists(m_dsl::member.find(id)))
            .get_result::<bool>(conn)
            .map_err(DbError::from)
    }

    /// Creation =============================
    pub(crate) fn create(conn: &mut DbConn, new_member: NewMember) -> Result<MemberRow, DbError> {
        diesel::insert_into(m_dsl::member)
            .values(&new_member)
            .get_result::<MemberRow>(conn)
            .map_err(DbError::from)
    }

    /// Reading Data =============================
    pub(crate) fn read(conn: &mut DbConn, id: MemberId) -> Result<Option<MemberRow>, DbError> {
        m_dsl::member
            .find(id)
            .first::<MemberRow>(conn)
            .optional()
            .map_err(DbError::from)
    }

    pub(crate) fn read_by_username(
        conn: &mut DbConn,
        username: &str,
    ) -> Result<Option<MemberRow>, DbError> {
        m_dsl::member
            .filter(m_dsl::username.eq(username))
            .first::<MemberRow>(conn)
            .optional()
            .map_err(DbError::from)
    }

    /// Reads the profile-safe projection for a member.
    pub(crate) fn read_detailed(
        conn: &mut DbConn,
        id: MemberId,
    ) -> Result<Option<MemberDetailed>, DbError> {
        m_dsl::member
            .find(id)
            .select(MemberDetailedRow::as_select())
            .first(conn)
            .optional()
            .map(|member| member.map(MemberDetailed::from))
            .map_err(DbError::from)
    }

    pub(crate) fn search_usernames(
        conn: &mut DbConn,
        search: SearchValidation,
    ) -> Result<Vec<MemberShort>, DbError> {
        let prefix = format!("{}%", search.escaped);
        let contains = format!("%{}%", search.escaped);

        let relevance = case_when::<_, _, Integer>(m_dsl::username.eq(search.normalized), 0)
            .when(m_dsl::username.like(&prefix).escape('\\'), 1)
            .when(m_dsl::username.ilike(&prefix).escape('\\'), 2)
            .otherwise(3);

        m_dsl::member
            .filter(m_dsl::username.ilike(&contains).escape('\\'))
            .order_by((relevance.asc(), m_dsl::username.asc()))
            .limit(25)
            .select(MemberShortRow::as_select())
            .load(conn)
            .map(|results: Vec<MemberShortRow>| {
                results.into_iter().map(MemberShort::from).collect()
            })
            .map_err(DbError::from)
    }
    // Updating Data =============================
    pub(super) fn update(
        conn: &mut DbConn,
        id: MemberId,
        entity: MemberChangeset,
    ) -> Result<MemberRow, DbError> {
        // Update member details by id
        diesel::update(m_dsl::member.filter(m_dsl::id.eq(id)))
            .set(entity)
            .get_result::<MemberRow>(conn)
            .map_err(DbError::from)
    }
    pub(crate) fn update_password(
        conn: &mut DbConn,
        id: MemberId,
        current_hash: &str,
        new_hash: String,
    ) -> Result<bool, DbError> {
        diesel::update(
            m_dsl::member
                .filter(m_dsl::id.eq(id))
                .filter(m_dsl::password_hash.eq(current_hash)),
        )
        .set(m_dsl::password_hash.eq(new_hash))
        .execute(conn)
        .map(|updated| updated == 1)
        .map_err(DbError::from)
    }

    // Deleting Data =============================
    pub(super) fn delete(conn: &mut DbConn, id: MemberId) -> Result<bool, DbError> {
        diesel::delete(m_dsl::member.find(id))
            .execute(conn)
            .map(|deleted| deleted == 1)
            .map_err(DbError::from)
    }
}
