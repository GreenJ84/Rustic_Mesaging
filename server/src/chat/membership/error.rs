use diesel::result::Error as DieselError;

use crate::{api_error::ApiError, database::DbError};

#[derive(Debug)]
pub(crate) enum ChatMembershipError {
    InvalidInput(String),

    NotFound,
    ChatNotFound,
    MemberNotFound,

    AlreadyMember,
    NotMember,

    PersistenceError(DbError),
}

impl From<DbError> for ChatMembershipError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,
            DbError::Conflict(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("member_chat_pkey") => Self::AlreadyMember,
                    _ => Self::PersistenceError(error),
                }
            }
            DbError::ConstraintViolation(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("member_chat_chat_id_fkey") => Self::ChatNotFound,
                    Some("member_chat_member_id_fkey") => Self::MemberNotFound,
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

impl From<DieselError> for ChatMembershipError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<ChatMembershipError> for ApiError {
    fn from(error: ChatMembershipError) -> Self {
        match error {
            ChatMembershipError::InvalidInput(message) => ApiError::BadRequest(message),
            ChatMembershipError::NotFound => ApiError::NotFound("Chat membership not found".into()),
            ChatMembershipError::ChatNotFound => ApiError::NotFound("Chat not found".into()),
            ChatMembershipError::MemberNotFound => ApiError::NotFound("Member not found".into()),
            ChatMembershipError::AlreadyMember => {
                ApiError::Conflict("Member is already in this chat".into())
            }
            ChatMembershipError::NotMember => {
                ApiError::NotFound("Chat membership not found".into())
            }
            ChatMembershipError::PersistenceError(error) => {
                log::error!("Chat membership persistence error: {error:?}");
                ApiError::InternalError
            }
        }
    }
}
