use diesel::Connection;

use crate::chat::models::MultiChatPreview;
use crate::server::models::MultiServer;
use crate::utils::SearchValidation;
use crate::{auth::system_roles::SystemRoleService, database::Database, utils::Validator};

use super::{
    error::{DeletionBlock, MemberError},
    models::{
        MemberChangeset, MemberId, MemberProfile, MemberShort, MemberUpdateInput,
        MultiMemberPreview,
    },
    repository::MemberRepository,
};

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
        let search = Validator::validate_search_term(search_term)
            .map_err(|e| MemberError::InvalidSearchTerm(format!("Members search term {}", e.message())))?;
        let mut conn = self.database.get_connection()?;
        Ok(MultiMemberPreview {
            members: MemberRepository::search_usernames(&mut conn, search)?,
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
