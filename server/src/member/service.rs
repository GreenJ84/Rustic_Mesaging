use diesel::Connection;

use crate::{auth::system_roles::SystemRoleService, infra::Database, utils::Validator};

use super::{
    error::{DeletionBlock, MemberError},
    models::{
        MemberChangeset, MemberDetailed, MemberId, MemberRow, MemberShort, MemberUpdateInput,
        MultiMemberPreview,
    },
    repository::MemberRepository,
};

pub(crate) struct MemberService {
    database: Database,
}

impl MemberService {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }

    // TODO(Critical): Require an authorized admin-level request guard.
    pub(super) fn get(&self, member_id: MemberId) -> Result<MemberRow, MemberError> {
        let mut conn = self.database.get_connection()?;
        if let Some(member) = MemberRepository::read(&mut conn, member_id)? {
            Ok(member)
        } else {
            Err(MemberError::NotFound)
        }
    }

    // TODO(Critical): Require an authorized request guard.
    pub(super) fn get_detailed(&self, member_id: MemberId) -> Result<MemberDetailed, MemberError> {
        let mut conn = self.database.get_connection()?;
        if let Some(member) = MemberRepository::read_detailed(&mut conn, member_id)? {
            Ok(member)
        } else {
            Err(MemberError::NotFound)
        }
    }

    pub(super) fn search(&self, search_term: String) -> Result<MultiMemberPreview, MemberError> {
        let search = Validator::validate_search_term(search_term).map_err(|e| {
            MemberError::InvalidSearchTerm(format!("Members search term {}", e.message()))
        })?;
        let mut conn = self.database.get_connection()?;
        Ok(MultiMemberPreview {
            members: MemberRepository::search_usernames(&mut conn, search)?,
        })
    }

    pub(super) fn update(
        &self,
        member_id: MemberId,
        input: MemberUpdateInput,
    ) -> Result<MemberDetailed, MemberError> {
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
            Ok(MemberDetailed::from(updated_member))
        })
    }

    // TODO(Severe): Move full account deletion behind the authenticated account lifecycle flow.
    pub(super) fn delete(&self, member_id: MemberId) -> Result<(), MemberError> {
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
}
