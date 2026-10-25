use diesel::Connection;

use crate::auth::system_roles::SystemRoleService;
use crate::database::Database;
use crate::member::error::DeletionBlock;
use crate::member::models::MemberUpdateInput;

use super::error::MemberError;
use super::models::{MemberChangeset, MemberId, MemberProfile, MemberShort, MultiMemberPreview};
use super::repository::MemberRepository;

use crate::chat::models::MultiChatPreview;
use crate::server::models::MultiServer;

pub struct MemberService {
    database: Database,
}

impl MemberService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Member creation ONLY through authenticating Registration route
    pub fn get(&self, member_id: MemberId) -> Result<MemberProfile, MemberError> {
        let mut conn = self.database.get_connection()?;
        if let Some(member) = MemberRepository::find_by_id(&mut conn, member_id)? {
            Ok(member.into_profile())
        } else {
            Err(MemberError::NotFound)
        }
    }
    pub fn find_by_username(&self, username: String) -> Result<Option<MemberShort>, MemberError> {
        let mut conn = self.database.get_connection()?;
        if let Some(member) = MemberRepository::find_by_username(&mut conn, &username)? {
            Ok(Some(member.into_short()))
        } else {
            Ok(None)
        }
    }
    pub fn search_members(&self, search_term: String) -> Result<MultiMemberPreview, MemberError> {
        let search_term = normalize_search_term(&search_term)?;
        let mut conn = self.database.get_connection()?;
        Ok(MultiMemberPreview {
            members: MemberRepository::search_usernames(&mut conn, search_term)?,
        })
    }

    pub fn update(
        &self,
        member_id: MemberId,
        input: MemberUpdateInput,
    ) -> Result<MemberProfile, MemberError> {
        let input = MemberChangeset::try_from(input)?;
        let mut conn = self.database.get_connection()?;
        conn.transaction(|conn| {
            if let Some(username) = MemberRepository::identity_exists_excluding(
                conn,
                member_id,
                input.username.as_deref().unwrap_or(""),
                input.email.as_deref().unwrap_or(""),
            )? {
                if username == input.username.as_deref().unwrap_or("") {
                    return Err(MemberError::UsernameTaken);
                } else {
                    return Err(MemberError::EmailTaken);
                }
            };
            let updated_member = MemberRepository::update(conn, member_id, input)?;
            Ok(updated_member.into_profile())
        })
    }

    pub fn delete(&self, member_id: MemberId) -> Result<(), MemberError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<(), MemberError, _>(|conn| {
            if !SystemRoleService::validate_member_deletion(conn, member_id)? {
                return Err(MemberError::DeletionBlocked(
                    DeletionBlock::LastAdministrator,
                ));
            }
            if MemberRepository::delete(conn, member_id)? {
                Ok(())
            } else {
                Err(MemberError::NotFound)
            }
        })
    }

    // TODO: Migrate into associated service for chats, servers, and friends
    pub fn get_member_chats(&self, member_id: MemberId) -> Result<MultiChatPreview, MemberError> {
        let mut conn = self.database.get_connection()?;
        Ok(MultiChatPreview {
            chat_previews: MemberRepository::get_member_chats(&mut conn, member_id)?,
        })
    }
}

fn normalize_search_term(search_term: &str) -> Result<String, MemberError> {
    let search_term = search_term.trim();
    if search_term.is_empty() {
        return Err(MemberError::InvalidSearchTerm(
            "Search term cannot be empty".to_string(),
        ));
    }
    if search_term.chars().count() > 125 {
        return Err(MemberError::InvalidSearchTerm(
            "Search term cannot exceed 125 characters".to_string(),
        ));
    }
    Ok(search_term.to_owned())
}

#[cfg(test)]
mod tests {
    use super::normalize_search_term;
    use crate::member::error::MemberError;

    #[test]
    fn normalizes_search_terms() {
        assert_eq!(normalize_search_term("  Alice  ").unwrap(), "Alice");
    }

    #[test]
    fn rejects_empty_search_terms() {
        assert!(matches!(
            normalize_search_term(" \t\n "),
            Err(MemberError::InvalidSearchTerm(_))
        ));
    }

    #[test]
    fn limits_search_terms_by_character_count() {
        assert!(normalize_search_term(&"é".repeat(125)).is_ok());
        assert!(matches!(
            normalize_search_term(&"é".repeat(126)),
            Err(MemberError::InvalidSearchTerm(_))
        ));
    }
}
