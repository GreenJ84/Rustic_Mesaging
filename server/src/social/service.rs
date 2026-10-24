use diesel::Connection;

use crate::database::{Database, DbError};
use crate::member::models::MemberId;
use crate::member::repository::MemberRepository;
use crate::social::error::SocialError;
use crate::social::friend::models::{Friendship, NewFriend};

use super::friend::models::{Friend, FriendList, FriendPreview};
use super::friend::repository::FriendRepository;
use super::request::models::{FriendRequest, MultiFriendRequest, NewFriendRequest};
use super::request::repository::{RequestRepository, RequestType};

pub enum RequestCreation {
    NewFriend(Friendship),
    NewRequest(FriendRequest),
}
pub(crate) struct SocialService {
    database: Database,
}
impl SocialService {
    pub fn new(database: Database) -> Self {
        SocialService { database }
    }

    // ================================
    // Request Lifecycle
    // ================================
    // Create
    pub(crate) fn send_friend_request(
        &self,
        member_id: MemberId,
        receiver_id: MemberId,
        note: String,
    ) -> Result<RequestCreation, SocialError> {
        if member_id == receiver_id {
            return Err(SocialError::SelfFriendship);
        }
        let new_request = NewFriendRequest {
            sender_id: member_id,
            receiver_id,
            note,
        };

        let mut conn = self.database.get_connection()?;
        conn.transaction::<RequestCreation, SocialError, _>(|conn| {
            if MemberRepository::find_by_id(conn, receiver_id)?.is_none() {
                return Err(SocialError::MemberNotFound);
            } else if FriendRepository::exists(conn, member_id, receiver_id)? {
                return Err(SocialError::AlreadyFriends);
            }
            match RequestRepository::is_requested(
                conn,
                new_request.sender_id,
                new_request.receiver_id,
            ) {
                Ok(RequestType::Incoming) => {
                    // If the request is incoming, accept it and create a friendship
                    let new_friendship =
                        NewFriend::new(new_request.sender_id, new_request.receiver_id);
                    let friendship = FriendRepository::create(conn, new_friendship)?;
                    RequestRepository::delete_request(
                        conn,
                        new_request.receiver_id,
                        new_request.sender_id,
                    )?;
                    Ok(RequestCreation::NewFriend(friendship))
                }
                Ok(RequestType::Outgoing) => {
                    // If the request is outgoing, return an error indicating that the request already exists
                    Err(SocialError::RequestAlreadyExists)
                }
                Err(DbError::NotFound) => {
                    // If no request exists, create a new friend request
                    let new_request = RequestRepository::create_request(conn, new_request)?;
                    Ok(RequestCreation::NewRequest(new_request))
                }
                Err(e) => Err(SocialError::from(e)),
            }
        })
    }

    // Get
    pub(crate) fn is_requested(
        &self,
        sender_id: MemberId,
        receiver_id: MemberId,
    ) -> Result<Option<bool>, SocialError> {
        let mut conn = self.database.get_connection()?;
        match RequestRepository::is_requested(&mut conn, sender_id, receiver_id) {
            Ok(RequestType::Incoming) => Ok(Some(true)),
            Ok(RequestType::Outgoing) => Ok(Some(false)),
            Err(DbError::NotFound) => Ok(None),
            Err(e) => Err(SocialError::from(e)),
        }
    }
    pub(crate) fn get_friend_request(
        &self,
        sender_id: MemberId,
        receiver_id: MemberId,
    ) -> Result<FriendRequest, SocialError> {
        let mut conn = self.database.get_connection()?;
        RequestRepository::get_request(&mut conn, sender_id, receiver_id).map_err(|err| match err {
            DbError::NotFound => SocialError::RequestNotFound,
            _ => SocialError::from(err),
        })
    }
    pub(crate) fn get_friend_requests(
        &self,
        member_id: MemberId,
    ) -> Result<MultiFriendRequest, SocialError> {
        let mut conn = self.database.get_connection()?;
        let incoming =
            RequestRepository::get_incoming(&mut conn, member_id).map_err(SocialError::from)?;
        let outgoing =
            RequestRepository::get_outgoing(&mut conn, member_id).map_err(SocialError::from)?;
        Ok(MultiFriendRequest { incoming, outgoing })
    }

    // Update
    pub(crate) fn accept_friend_request(
        &self,
        member_id: MemberId,
        sender_id: MemberId,
    ) -> Result<Friendship, SocialError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<Friendship, SocialError, _>(|conn| {
            let found = RequestRepository::is_requested(conn, member_id, sender_id).map_err(
                |err| match err {
                    DbError::NotFound => SocialError::RequestNotFound,
                    _ => SocialError::from(err),
                },
            )?;
            match found {
                RequestType::Incoming => {
                    let new_friendship = NewFriend::new(sender_id, member_id);
                    let friendship = FriendRepository::create(conn, new_friendship)?;
                    RequestRepository::delete_request(conn, sender_id, member_id)?;
                    Ok(friendship)
                }
                RequestType::Outgoing => Err(SocialError::NotRequestRecipient),
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
            let found = RequestRepository::is_requested(conn, member_id, sender_id).map_err(
                |err| match err {
                    DbError::NotFound => SocialError::RequestNotFound,
                    _ => SocialError::from(err),
                },
            )?;
            match found {
                RequestType::Incoming => {
                    RequestRepository::delete_request(conn, sender_id, member_id)?;
                    Ok(())
                }
                RequestType::Outgoing => Err(SocialError::NotRequestRecipient),
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
            let found =
                RequestRepository::is_requested(conn, member_id, receiver_id).map_err(|err| {
                    match err {
                        DbError::NotFound => SocialError::RequestNotFound,
                        _ => SocialError::from(err),
                    }
                })?;
            match found {
                RequestType::Incoming => Err(SocialError::NotRequestSender),
                RequestType::Outgoing => {
                    RequestRepository::delete_request(conn, member_id, receiver_id)?;
                    Ok(())
                }
            }
        })
    }
    // ================================
    // Friend Lifecycle
    // ================================
    // Create is internally handled by accepting a friend request, which creates a friendship record.

    // Get
    pub(crate) fn is_friend(
        &self,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<bool, SocialError> {
        let mut conn = self.database.get_connection()?;
        FriendRepository::exists(&mut conn, member_id, friend_id).map_err(|err| match err {
            DbError::NotFound => SocialError::FriendshipNotFound,
            _ => SocialError::from(err),
        })
    }
    pub(crate) fn get_friend(
        &self,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<Friend, SocialError> {
        let mut conn = self.database.get_connection()?;
        FriendRepository::find(&mut conn, member_id, friend_id).map_err(|err| match err {
            DbError::NotFound => SocialError::FriendshipNotFound,
            _ => SocialError::from(err),
        })
    }
    pub(crate) fn get_member_friends(
        &self,
        member_id: MemberId,
    ) -> Result<FriendList<Friend>, SocialError> {
        let mut conn = self.database.get_connection()?;
        FriendRepository::find_multi(&mut conn, member_id).map_err(SocialError::from)
    }
    pub(crate) fn get_member_friend_previews(
        &self,
        member_id: MemberId,
    ) -> Result<FriendList<FriendPreview>, SocialError> {
        let mut conn = self.database.get_connection()?;
        FriendRepository::find_multi_preview(&mut conn, member_id).map_err(SocialError::from)
    }
    // Delete
    pub(crate) fn remove_friend(
        &self,
        member_id: MemberId,
        friend_id: MemberId,
    ) -> Result<(), SocialError> {
        let mut conn = self.database.get_connection()?;
        match FriendRepository::delete(&mut conn, member_id, friend_id) {
            Ok(size) => {
                if size == 0 {
                    Err(SocialError::FriendshipNotFound)
                } else {
                    Ok(())
                }
            }
            Err(DbError::NotFound) => Err(SocialError::FriendshipNotFound),
            Err(e) => Err(SocialError::Persistence(e)),
        }
    }
}
