use chrono::NaiveDateTime;
use diesel::dsl::case_when;
use diesel::prelude::*;
use diesel::sql_types::Integer;

use crate::chat::models::Chat;
use crate::database::{DbConn, DbError};
use crate::member::models::{Member, MemberChangeset, MemberId, MemberShort, NewMember};
use crate::schema::{
    chat::dsl as c_dsl, friend::dsl as f_dsl, friend_request::dsl as fr_dsl, member::dsl as m_dsl,
    member_chat::dsl as mc_dsl, server::dsl as s_dsl, server_membership::dsl as sm_dsl,
};

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
    pub(crate) fn create(conn: &mut DbConn, new_member: NewMember) -> Result<Member, DbError> {
        diesel::insert_into(m_dsl::member)
            .values(&new_member)
            .get_result::<Member>(conn)
            .map_err(DbError::from)
    }

    /// Reading Data =============================
    fn read(conn: &mut DbConn, identifier: MemberIdentifier) -> Result<Option<Member>, DbError> {
        match identifier {
            MemberIdentifier::Id(id) => m_dsl::member.find(id).first::<Member>(conn).optional(),
            MemberIdentifier::Username(username) => m_dsl::member
                .filter(m_dsl::username.eq(username))
                .first::<Member>(conn)
                .optional(),
        }
        .map_err(DbError::from)
    }
    pub fn find_by_id(conn: &mut DbConn, id: MemberId) -> Result<Option<Member>, DbError> {
        Self::read(conn, MemberIdentifier::Id(id))
    }
    pub(crate) fn find_by_username(
        conn: &mut DbConn,
        username: &str,
    ) -> Result<Option<Member>, DbError> {
        Self::read(conn, MemberIdentifier::Username(username))
    }
    pub fn search_usernames(
        conn: &mut DbConn,
        search_term: String,
    ) -> Result<Vec<MemberShort>, DbError> {
        let escaped = Self::escape_like_pattern(&search_term);

        let prefix = format!("{escaped}%");
        let contains = format!("%{escaped}%");

        let relevance = case_when::<_, _, Integer>(m_dsl::username.eq(search_term), 0)
            .when(m_dsl::username.like(&prefix).escape('\\'), 1)
            .when(m_dsl::username.ilike(&prefix).escape('\\'), 2)
            .otherwise(3);

        m_dsl::member
            .filter(m_dsl::username.ilike(&contains).escape('\\'))
            .order_by((relevance.asc(), m_dsl::username.asc()))
            .limit(25)
            .select(MemberShort::as_select())
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

    // Updating Data =============================
    pub(super) fn update(
        conn: &mut DbConn,
        id: MemberId,
        entity: MemberChangeset,
    ) -> Result<Member, DbError> {
        // Update member details by id
        diesel::update(m_dsl::member.filter(m_dsl::id.eq(id)))
            .set(entity)
            .get_result::<Member>(conn)
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

    // TODO: Move to sub-services ===============
    pub fn get_member_chats(conn: &mut DbConn, member_id: i32) -> Result<Vec<Chat>, DbError> {
        mc_dsl::member_chat
            .filter(mc_dsl::member_id.eq(member_id))
            .inner_join(c_dsl::chat.on(mc_dsl::chat_id.eq(c_dsl::id)))
            .select(Chat::as_select())
            .load::<Chat>(conn)
            .map_err(DbError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::MemberRepository;

    #[test]
    fn escapes_like_wildcards_and_escape_characters() {
        assert_eq!(
            MemberRepository::escape_like_pattern(r"50%_off\today"),
            r"50\%\_off\\today"
        );
    }

    #[test]
    fn leaves_plain_search_terms_unchanged() {
        assert_eq!(MemberRepository::escape_like_pattern("alice"), "alice");
    }
}
