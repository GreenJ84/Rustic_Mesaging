use diesel::Connection;
use serde::{Deserialize, Serialize};

use crate::database::{Database, DbError};
use crate::member::models::MemberId;
use crate::member::repository::MemberRepository;
use crate::social::error::SocialError;

use super::friend::models::{Friend, FriendList, FriendPreview, NewFriend};
use super::friend::repository::FriendRepository;
use super::request::models::{FriendRequest, MultiFriendRequest, NewFriendRequest, RequestDetails};
use super::request::repository::{RequestRepository, RequestType};

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestCreation {
    NewFriend(Friend),
    NewRequest(RequestDetails),
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
                    FriendRepository::create(conn, new_friendship)?;
                    RequestRepository::delete_request(
                        conn,
                        new_request.receiver_id,
                        new_request.sender_id,
                    )?;
                    let friendship = FriendRepository::find(
                        conn,
                        new_request.sender_id,
                        new_request.receiver_id,
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
                    let new_request = RequestRepository::get_request(
                        conn,
                        new_request.sender_id(),
                        new_request.receiver_id(),
                    )?;
                    Ok(RequestCreation::NewRequest(new_request))
                }
                Err(e) => Err(SocialError::from(e)),
            }
        })
    }

    // Get
    pub(crate) fn get_friend_request(
        &self,
        sender_id: MemberId,
        receiver_id: MemberId,
    ) -> Result<RequestDetails, SocialError> {
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
    pub(crate) fn update_request(
        &self,
        member_id: MemberId,
        receiver_id: MemberId,
        note: String,
    ) -> Result<(), SocialError> {
        let mut conn = self.database.get_connection()?;
        let found =
            RequestRepository::is_requested(&mut conn, member_id, receiver_id).map_err(|err| {
                match err {
                    DbError::NotFound => SocialError::RequestNotFound,
                    _ => SocialError::from(err),
                }
            })?;
        match found {
            RequestType::Incoming => Err(SocialError::NotRequestSender),
            RequestType::Outgoing => {
                let updated_request = NewFriendRequest {
                    sender_id: member_id,
                    receiver_id,
                    note,
                };
                let updated_request = RequestRepository::update_request(&mut conn, updated_request)
                    .map_err(|err| match err {
                        DbError::NotFound => SocialError::RequestNotFound,
                        _ => SocialError::from(err),
                    })?;
                Ok(())
            }
        }
    }
    pub(crate) fn accept_friend_request(
        &self,
        member_id: MemberId,
        sender_id: MemberId,
    ) -> Result<Friend, SocialError> {
        let mut conn = self.database.get_connection()?;
        conn.transaction::<Friend, SocialError, _>(|conn| {
            let found = RequestRepository::is_requested(conn, member_id, sender_id).map_err(
                |err| match err {
                    DbError::NotFound => SocialError::RequestNotFound,
                    _ => SocialError::from(err),
                },
            )?;
            match found {
                RequestType::Incoming => {
                    let new_friendship = NewFriend::new(sender_id, member_id);
                    FriendRepository::create(conn, new_friendship)?;
                    RequestRepository::delete_request(conn, sender_id, member_id)?;
                    let friend = FriendRepository::find(conn, sender_id, member_id)?;
                    Ok(friend)
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
                    RequestRepository::delete_request(conn, sender_id, member_id).map(|size| {
                        if size < 1 {
                            Err(SocialError::RequestNotFound)
                        } else {
                            Ok(())
                        }
                    })?;
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
