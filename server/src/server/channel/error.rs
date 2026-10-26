use diesel::result::Error as DieselError;

use crate::api_error::ApiError;
use crate::database::DbError;

#[derive(Debug)]
pub enum ChannelError {
    InvalidInput(String),
    InvalidChangeset(String),

    NotFound,
    NameTaken,

    ServerNotFound,
    MembershipRequired,
    PermissionDenied,

    PersistenceError(DbError),
}
impl From<DbError> for ChannelError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,

            DbError::Conflict(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("channel_server_name_unique") => Self::NameTaken,
                    Some("channel_server_id_fkey") => Self::ServerNotFound,
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
impl From<DieselError> for ChannelError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<ChannelError> for ApiError {
    fn from(error: ChannelError) -> Self {
        match error {
            ChannelError::InvalidInput(message) => ApiError::BadRequest(message),
            ChannelError::InvalidChangeset(message) => ApiError::BadRequest(message),

            ChannelError::NotFound => ApiError::NotFound("Channel not found".into()),
            ChannelError::NameTaken => {
                ApiError::Conflict("Channel name is already taken for the associated server".into())
            }

            ChannelError::ServerNotFound => {
                ApiError::NotFound("Associated server not found".into())
            }
            ChannelError::MembershipRequired => ApiError::Forbidden(
                "You do not have the required membership authority for this operation".into(),
            ),
            ChannelError::PermissionDenied => ApiError::Forbidden(
                "You do not have the required server permission for this operation".into(),
            ),

            ChannelError::PersistenceError(error) => {
                log::error!("Channel persistence error: {error:?}");
                ApiError::InternalError
            }
        }
    }
}
