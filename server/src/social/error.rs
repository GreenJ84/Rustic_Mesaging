use diesel::result::Error as DieselError;

use crate::api::ApiError;
use crate::infra::DbError;

#[derive(Debug)]
pub(crate) enum SocialError {
    InvalidInput(String),
    SelfFriendship,
    MemberNotFound,

    FriendshipNotFound,
    AlreadyFriends,

    RequestNotFound,
    RequestAlreadyExists,
    NotRequestRecipient,
    NotRequestSender,

    Persistence(DbError),
}
impl From<DbError> for SocialError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::Conflict(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("friend_pkey") => SocialError::AlreadyFriends,

                    Some("friend_request_pkey") | Some("friend_request_pair_unique") => {
                        SocialError::RequestAlreadyExists
                    }
                    Some("friend_request_already_friend") => SocialError::AlreadyFriends,

                    _ => SocialError::Persistence(error),
                }
            }
            DbError::ConstraintViolation(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("friend_members_must_differ") => SocialError::SelfFriendship,
                    Some("friend_ids_are_canonical") => {
                        SocialError::InvalidInput("Friend IDs must be in ascending order".into())
                    }

                    Some("friend_request_members_must_differ") => SocialError::SelfFriendship,

                    _ => SocialError::Persistence(error),
                }
            }
            _ => Self::Persistence(error),
        }
    }
}
impl From<DieselError> for SocialError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<SocialError> for ApiError {
    fn from(error: SocialError) -> Self {
        match error {
            SocialError::InvalidInput(message) => ApiError::BadRequest(message),
            SocialError::SelfFriendship => {
                ApiError::BadRequest("You cannot befriend yourself".into())
            }
            SocialError::MemberNotFound => ApiError::NotFound("Member not found".into()),
            SocialError::FriendshipNotFound => ApiError::NotFound("Friendship not found".into()),
            SocialError::RequestNotFound => ApiError::NotFound("Friend request not found".into()),
            SocialError::AlreadyFriends => ApiError::Conflict("Friendship already exists".into()),
            SocialError::RequestAlreadyExists => {
                ApiError::Conflict("Friend request already exists".into())
            }
            SocialError::NotRequestRecipient | SocialError::NotRequestSender => {
                ApiError::Forbidden("You cannot perform this action".into())
            }
            SocialError::Persistence(_) => ApiError::InternalError,
        }
    }
}
