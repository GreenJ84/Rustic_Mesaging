use diesel::Connection;

use crate::infra::Database;
use crate::member::models::MemberId;
use crate::member::repository::MemberRepository;
use crate::social::error::SocialError;
use crate::social::friend::models::NewFriend;
use crate::social::friend::repository::FriendRepository;
use crate::social::request::models::{FriendRequestChangeset, NewFriendRequest};
use crate::social::request::repository::{FriendRequestDirection, FriendRequestRepository};
use crate::utils::Validator;
use shared::models::social::friend::{FriendDetails, MultiFriend, MultiFriendPreview};
use shared::models::social::request::{
    FriendRequestChangesetInput, FriendRequestCreation, FriendRequestDetailed, MultiFriendRequest,
    NewFriendRequestInput,
};

pub(crate) struct SocialService {
    database: Database,
}

impl SocialService {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }

    pub(crate) fn send_friend_request(
        &self,
        member_id: MemberId,
        input: NewFriendRequestInput,
    ) -> Result<FriendRequestCreation, SocialError> {
        if member_id == input.receiver_id {
            return Err(SocialError::SelfFriendship);
        }

        let receiver_id = input.receiver_id;
        let new_request = NewFriendRequest {
            sender_id: member_id,
            receiver_id,
            note: normalize_note(input.note)?,
        };

        let mut conn = self.database.get_connection()?;
        conn.transaction::<FriendRequestCreation, SocialError, _>(|conn| {
            if !MemberRepository::exists(conn, receiver_id)? {
                return Err(SocialError::MemberNotFound);
            }
            if FriendRepository::exists(conn, member_id, receiver_id)? {
                return Err(SocialError::AlreadyFriends);
            }

            match FriendRequestRepository::read_direction_for_update(conn, member_id, receiver_id)?
            {
                Some(FriendRequestDirection::Incoming) => {
                    FriendRepository::create(conn, NewFriend::new(member_id, receiver_id))?;
                    if !FriendRequestRepository::delete(conn, receiver_id, member_id)? {
                        return Err(SocialError::RequestNotFound);
                    }
                    let friend = FriendRepository::read_detailed(conn, member_id, receiver_id)?
                        .ok_or(SocialError::FriendshipNotFound)?;
                    Ok(FriendRequestCreation::Friend(FriendDetails::from(friend)))
                }
                Some(FriendRequestDirection::Outgoing) => Err(SocialError::RequestAlreadyExists),
                None => {
                    FriendRequestRepository::create(conn, new_request)?;
                    let request =
                        FriendRequestRepository::read_detailed(conn, member_id, receiver_id)?
                            .ok_or(SocialError::RequestNotFound)?;
                    Ok(FriendRequestCreation::Request(FriendRequestDetailed::from(
                        request,
                    )))
                }
            }
        })
    }

    pub(crate) fn get_friend_request(
        &self,
        member_id: MemberId,
        other_member_id: MemberId,
    ) -> Result<FriendRequestDetailed, SocialError> {
        let mut conn = self.database.get_connection()?;
        FriendRequestRepository::read_detailed(&mut conn, member_id, other_member_id)?
            .map(FriendRequestDetailed::from)
            .ok_or(SocialError::RequestNotFound)
    }

    pub(crate) fn get_friend_requests(
        &self,
        member_id: MemberId,
    ) -> Result<MultiFriendRequest, SocialError> {
        let mut conn = self.database.get_connection()?;
        let incoming = FriendRequestRepository::read_incoming(&mut conn, member_id)?
            .into_iter()
            .map(FriendRequestDetailed::from)
            .collect();
        let outgoing = FriendRequestRepository::read_outgoing(&mut conn, member_id)?
            .into_iter()
            .map(FriendRequestDetailed::from)
            .collect();
        Ok(MultiFriendRequest { incoming, outgoing })
    }

    pub(crate) fn update_request(
        &self,
        member_id: MemberId,
        receiver_id: MemberId,
        input: FriendRequestChangesetInput,
    ) -> Result<(), SocialError> {
        let changeset = FriendRequestChangeset {
            note: normalize_note(input.note)?,
        };
        let mut conn = self.database.get_connection()?;
        conn.transaction::<(), SocialError, _>(|conn| {
            match FriendRequestRepository::read_direction_for_update(conn, member_id, receiver_id)?
            {
                Some(FriendRequestDirection::Incoming) => Err(SocialError::NotRequestSender),
                Some(FriendRequestDirection::Outgoing) => {
                    FriendRequestRepository::update(conn, member_id, receiver_id, changeset)?
                        .ok_or(SocialError::RequestNotFound)?;
                    Ok(())
                }
                None => Err(SocialError::RequestNotFound),
            }
        })
    }

    pub(crate) fn accept_friend_request(
        &self,
        member_id: MemberId,
        sender_id: MemberId,
    ) -> Result<FriendDetails, SocialError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<FriendDetails, SocialError, _>(|conn| {
            match FriendRequestRepository::read_direction_for_update(conn, member_id, sender_id)? {
                Some(FriendRequestDirection::Incoming) => {
                    FriendRepository::create(conn, NewFriend::new(sender_id, member_id))?;
                    if !FriendRequestRepository::delete(conn, sender_id, member_id)? {
                        return Err(SocialError::RequestNotFound);
                    }
                    FriendRepository::read_detailed(conn, member_id, sender_id)?
                        .map(FriendDetails::from)
                        .ok_or(SocialError::FriendshipNotFound)
                }
                Some(FriendRequestDirection::Outgoing) => Err(SocialError::NotRequestRecipient),
                None => Err(SocialError::RequestNotFound),
            }
        })
    }

    pub(crate) fn deny_friend_request(
        &self,
        member_id: MemberId,
        sender_id: MemberId,
    ) -> Result<(), SocialError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<(), SocialError, _>(|conn| {
            match FriendRequestRepository::read_direction_for_update(conn, member_id, sender_id)? {
                Some(FriendRequestDirection::Incoming) => {
                    if FriendRequestRepository::delete(conn, sender_id, member_id)? {
                        Ok(())
                    } else {
                        Err(SocialError::RequestNotFound)
                    }
                }
                Some(FriendRequestDirection::Outgoing) => Err(SocialError::NotRequestRecipient),
                None => Err(SocialError::RequestNotFound),
            }
        })
    }

    pub(crate) fn cancel_friend_request(
        &self,
        member_id: MemberId,
        receiver_id: MemberId,
    ) -> Result<(), SocialError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<(), SocialError, _>(|conn| {
            match FriendRequestRepository::read_direction_for_update(conn, member_id, receiver_id)?
            {
                Some(FriendRequestDirection::Incoming) => Err(SocialError::NotRequestSender),
                Some(FriendRequestDirection::Outgoing) => {
                    if FriendRequestRepository::delete(conn, member_id, receiver_id)? {
                        Ok(())
                    } else {
                        Err(SocialError::RequestNotFound)
                    }
                }
                None => Err(SocialError::RequestNotFound),
            }
        })
    }

    pub(crate) fn is_friend(
        &self,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<bool, SocialError> {
        let mut conn = self.database.get_connection()?;
        FriendRepository::exists(&mut conn, member_id, friend_id).map_err(SocialError::from)
    }

    pub(crate) fn get_friend(
        &self,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<FriendDetails, SocialError> {
        let mut conn = self.database.get_connection()?;
        FriendRepository::read_detailed(&mut conn, member_id, friend_id)?
            .map(FriendDetails::from)
            .ok_or(SocialError::FriendshipNotFound)
    }

    pub(crate) fn get_member_friends(
        &self,
        member_id: MemberId,
    ) -> Result<MultiFriend, SocialError> {
        let mut conn = self.database.get_connection()?;
        let friends = FriendRepository::read_many_detailed(&mut conn, member_id)?
            .into_iter()
            .map(FriendDetails::from)
            .collect();
        Ok(MultiFriend { friends })
    }

    pub(crate) fn get_member_friend_previews(
        &self,
        member_id: MemberId,
    ) -> Result<MultiFriendPreview, SocialError> {
        let mut conn = self.database.get_connection()?;
        let friends = FriendRepository::read_many_short(&mut conn, member_id)?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(MultiFriendPreview { friends })
    }

    pub(crate) fn remove_friend(
        &self,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<(), SocialError> {
        let mut conn = self.database.get_connection()?;
        if FriendRepository::delete(&mut conn, member_id, friend_id)? {
            Ok(())
        } else {
            Err(SocialError::FriendshipNotFound)
        }
    }
}

fn normalize_note(note: Option<String>) -> Result<Option<String>, SocialError> {
    let Some(note) = note else {
        return Ok(None);
    };
    if note.trim().is_empty() {
        return Ok(None);
    }
    Validator::validate_text(&note, Some(1), Some(500))
        .map(Some)
        .map_err(|error| {
            SocialError::InvalidInput(format!("Friend request note {}", error.message()))
        })
}

#[cfg(test)]
mod tests {
    use super::normalize_note;
    use crate::social::error::SocialError;

    #[test]
    fn normalizes_optional_request_notes() {
        assert_eq!(normalize_note(None).unwrap(), None);
        assert_eq!(normalize_note(Some("   ".into())).unwrap(), None);
        assert_eq!(
            normalize_note(Some("  Let's connect  ".into())).unwrap(),
            Some("Let's connect".into())
        );
    }

    #[test]
    fn rejects_request_notes_over_the_limit() {
        assert!(matches!(
            normalize_note(Some("a".repeat(501))),
            Err(SocialError::InvalidInput(_))
        ));
    }
}
