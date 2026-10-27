use diesel::result::Error as DieselError;

use crate::{api_error::ApiError, database::DbError};

use super::models::Chat;

#[derive(Debug)]
pub(crate) enum ChatError {
    InvalidInput(String),
    InvalidChangeset(String),

    NotFound,
    MemberNotFound,

    AlreadyMember,
    NotMember,

    PersistenceError(DbError),
}

impl From<DbError> for ChatError {
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

impl From<ChatError> for ApiError {
    fn from(error: ChatError) -> Self {
        match error {
            ChatError::InvalidInput(message) | ChatError::InvalidChangeset(message) => {
                ApiError::BadRequest(message)
            }
            ChatError::NotFound => ApiError::NotFound("Chat not found".into()),
            ChatError::MemberNotFound => ApiError::NotFound("Member not found".into()),

            ChatError::AlreadyMember => ApiError::Conflict("Member is already in this chat".into()),
            ChatError::NotMember => ApiError::NotFound("Chat membership not found".into()),
            ChatError::PersistenceError(error) => {
                log::error!("Chat persistence error: {error:?}");
                ApiError::InternalError
            }
        }
    }
}
