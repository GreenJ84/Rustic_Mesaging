use diesel::result::Error as DieselError;

use crate::{
    api_error::ApiError,
    chat::{membership::error::ChatMembershipError, models::Chat},
    database::DbError,
};

#[derive(Debug)]
pub(crate) enum ChatError {
    InvalidInput(String),
    InvalidChangeset(String),

    NotFound,
    MemberNotFound,

    PermissionDenied,

    PersistenceError(DbError),
}

impl From<DbError> for ChatError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,
            DbError::ConstraintViolation(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("member_chat_member_id_fkey") => Self::MemberNotFound,
                    Some("member_chat_chat_id_fkey") => Self::NotFound,
                    Some("chat_name_not_blank") => {
                        Self::InvalidInput("Chat name cannot be blank".into())
                    }
                    Some("chat_name_length") => {
                        Self::InvalidInput("Chat name must be between 3 and 80 characters".into())
                    }
                    _ => Self::PersistenceError(error),
                }
            }
            DbError::Conflict(_) | DbError::ConstraintViolation(_) => Self::PersistenceError(error),
            DbError::RetryableTransaction(_)
            | DbError::Serialization(_)
            | DbError::QueryBuilder(_)
            | DbError::Transaction(_)
            | DbError::Database(_)
            | DbError::ConnectionError(_)
            | DbError::PoolError(_) => Self::PersistenceError(error),
        }
    }
}
impl From<DieselError> for ChatError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<ChatMembershipError> for ChatError {
    fn from(error: ChatMembershipError) -> Self {
        match error {
            ChatMembershipError::InvalidInput(msg) => Self::InvalidInput(msg),

            ChatMembershipError::NotFound => Self::NotFound,
            ChatMembershipError::MemberNotFound => Self::MemberNotFound,
            ChatMembershipError::ChatNotFound => Self::NotFound,

            ChatMembershipError::NotMember => Self::PermissionDenied,
            ChatMembershipError::AlreadyMember => {
                Self::InvalidInput("Member is already in the chat".into())
            }


            ChatMembershipError::PersistenceError(db_error) => Self::PersistenceError(db_error),
        }
    }
}

impl From<ChatError> for ApiError {
    fn from(error: ChatError) -> Self {
        match error {
            ChatError::InvalidInput(message) | ChatError::InvalidChangeset(message) => {
                ApiError::BadRequest(message)
            }
            ChatError::NotFound => ApiError::NotFound("Chat not found".into()),
            ChatError::MemberNotFound => ApiError::NotFound("Member not found".into()),
            ChatError::PermissionDenied => {
                ApiError::Forbidden("You do not have permission to modify this chat".into())
            }
            ChatError::PersistenceError(error) => {
                log::error!("Chat persistence error: {error:?}");
                ApiError::InternalError
            }
        }
    }
}
