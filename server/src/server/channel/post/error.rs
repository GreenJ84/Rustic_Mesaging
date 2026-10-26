use diesel::result::Error as DieselError;

use crate::{api_error::ApiError, database::DbError};

#[derive(Debug)]
pub(crate) enum PostError {
    InvalidInput(String),
    InvalidChangeset(String),

    NotFound,

    ServerNotFound,
    ChannelNotFound,
    MemberNotFound,
    MembershipRequired,
    PermissionDenied,

    PersistenceError(DbError),
}
impl From<DbError> for PostError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,

            DbError::ConstraintViolation(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("post_channel_id_fkey") => Self::ChannelNotFound,
                    Some("post_author_id_fkey") => Self::MemberNotFound,
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
impl From<DieselError> for PostError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<PostError> for ApiError {
    fn from(error: PostError) -> Self {
        match error {
            PostError::InvalidInput(message) => ApiError::BadRequest(message),
            PostError::InvalidChangeset(message) => ApiError::BadRequest(message),

            PostError::NotFound => ApiError::NotFound("Post not found".into()),

            PostError::ServerNotFound => ApiError::NotFound("Server not found".into()),
            PostError::ChannelNotFound => ApiError::NotFound("Channel not found".into()),
            PostError::MemberNotFound => ApiError::NotFound("Member not found".into()),
            PostError::MembershipRequired => {
                ApiError::Forbidden("Membership required to perform this action".into())
            }
            PostError::PermissionDenied => {
                ApiError::Forbidden("You do not have permission to perform this action".into())
            }

            PostError::PersistenceError(_) => ApiError::InternalError,
        }
    }
}
