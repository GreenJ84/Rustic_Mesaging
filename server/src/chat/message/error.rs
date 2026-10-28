use crate::api::ApiError;
use crate::infra::{DbError, DieselError};

#[derive(Debug)]
pub(crate) enum MessageError {
    InvalidInput(String),
    InvalidChangeset(String),

    NotFound,
    ChatNotFound,

    MembershipRequired,
    PermissionDenied,
    SenderNotFound,

    PersistenceError(DbError),
}

impl From<DbError> for MessageError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,
            DbError::ConstraintViolation(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("message_chat_id_fkey") => Self::ChatNotFound,
                    Some("message_sender_id_fkey") => Self::SenderNotFound,
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

impl From<DieselError> for MessageError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<MessageError> for ApiError {
    fn from(error: MessageError) -> Self {
        match error {
            MessageError::InvalidInput(message) | MessageError::InvalidChangeset(message) => {
                ApiError::BadRequest(message)
            }
            MessageError::NotFound => ApiError::NotFound("Message not found".into()),
            MessageError::ChatNotFound => ApiError::NotFound("Chat not found".into()),
            MessageError::SenderNotFound => ApiError::NotFound("Member not found".into()),
            MessageError::MembershipRequired | MessageError::PermissionDenied => {
                ApiError::Forbidden("You cannot perform this message action".into())
            }
            MessageError::PersistenceError(error) => {
                log::error!("Message persistence error: {error:?}");
                ApiError::InternalError
            }
        }
    }
}
