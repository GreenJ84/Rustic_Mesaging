use diesel::Connection;

use crate::auth::system_roles::SystemRoleService;
use crate::db::{Db, DbPool};

use super::error::MemberError;
use super::models::{MemberChangeset, MemberId, MemberProfile, MemberShort, MultiMemberPreview};
use super::repository::MemberRepository;

use crate::chat::models::MultiChatPreview;
use crate::models::friend::FullFriend;
use crate::models::friend_request::RequestDetailed;
use crate::models::{friend::MultiFriend, friend_request::MultiFriendRequest};
use crate::server::models::MultiServer;

pub struct MemberService {
    pool: DbPool,
}

impl MemberService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Member creation ONLY through authenticating Registration route
    pub fn get(&self, member_id: MemberId) -> Result<MemberProfile, MemberError> {
        let mut conn = Db::connect(&self.pool)?;
        if let Some(member) = MemberRepository::find_by_id(&mut conn, member_id)? {
            Ok(member.into_profile())
        } else {
            Err(MemberError::NotFound)
        }
    }
    pub fn find_by_id(&self, id: MemberId) -> Result<Option<MemberShort>, MemberError> {
        let mut conn = Db::connect(&self.pool)?;
        if let Some(member) = MemberRepository::find_by_id(&mut conn, id)? {
            Ok(Some(member.into_short()))
        } else {
            Ok(None)
        }
    }
    pub fn find_by_username(&self, username: String) -> Result<Option<MemberShort>, MemberError> {
        let mut conn = Db::connect(&self.pool)?;
        if let Some(member) = MemberRepository::find_by_username(&mut conn, &username)? {
            Ok(Some(member.into_short()))
        } else {
            Ok(None)
        }
    }
    pub fn search_members(&self, search_term: String) -> Result<MultiMemberPreview, MemberError> {
        let search_term = search_term.trim();
        if search_term.is_empty() {
            return Err(MemberError::InvalidSearchTerm(
                "Search term cannot be empty".to_string(),
            ));
        }
        let mut conn = Db::connect(&self.pool)?;
        Ok(MultiMemberPreview {
            members: MemberRepository::search_usernames(&mut conn, search_term.to_owned())?,
        })
    }

    pub fn update(
        &self,
        member_id: MemberId,
        input: MemberChangeset,
    ) -> Result<MemberProfile, MemberError> {
        let input = input.normalize_and_validate()?;
        let mut conn = Db::connect(&self.pool)?;
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
        let mut conn = Db::connect(&self.pool)?;
        conn.transaction::<(), MemberError, _>(|conn| {
            if !SystemRoleService::validate_member_deletion(conn, member_id)? {
                return Err(MemberError::CannotDeleteLastAdministrator);
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
        let mut conn = Db::connect(&self.pool)?;
        Ok(MultiChatPreview {
            chat_previews: MemberRepository::get_member_chats(&mut conn, member_id)?,
        })
    }

    pub fn get_all_member_servers(&self, member_id: MemberId) -> Result<MultiServer, MemberError> {
        let mut conn = Db::connect(&self.pool)?;
        Ok(MultiServer {
            servers: MemberRepository::get_member_servers(&mut conn, member_id)?,
        })
    }

    pub fn get_member_friends(&self, member_id: MemberId) -> Result<MultiFriend, MemberError> {
        let mut conn = Db::connect(&self.pool)?;
        Ok(MultiFriend {
            friends: MemberRepository::get_member_friends(&mut conn, member_id)?
                .into_iter()
                .map(|(created_at, member)| FullFriend { member, created_at })
                .collect::<Vec<FullFriend>>(),
        })
    }

    pub fn get_friend_requests(
        &self,
        member_id: MemberId,
    ) -> Result<MultiFriendRequest, MemberError> {
        let mut conn = Db::connect(&self.pool)?;
        let (incoming, outgoing) = MemberRepository::get_member_requests(&mut conn, member_id)?;
        Ok(MultiFriendRequest {
            incoming: incoming
                .into_iter()
                .map(|entry| RequestDetailed {
                    member: entry.2,
                    note: entry.1,
                    created_at: entry.0,
                })
                .collect::<Vec<RequestDetailed>>(),
            outgoing: outgoing
                .into_iter()
                .map(|entry| RequestDetailed {
                    member: entry.2,
                    note: entry.1,
                    created_at: entry.0,
                })
                .collect::<Vec<RequestDetailed>>(),
        })
    }
}
