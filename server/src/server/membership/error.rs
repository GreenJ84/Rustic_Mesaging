use crate::api::ApiError;
use crate::infra::{DbError, DieselError};

#[derive(Debug)]
pub(crate) enum MembershipError {
    MembershipNotFound,
    MembershipRequired,
    AlreadyMember,

    OwnerCannotLeave,

    Persistence(DbError),
}
impl From<DbError> for MembershipError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::MembershipNotFound,

            DbError::Conflict(DieselError::DatabaseError(_, info)) => {
                match info.constraint_name() {
                    Some("server_membership_pkey") => Self::AlreadyMember,
                    _ => Self::Persistence(error),
                }
            }

            DbError::Conflict(_)
            | DbError::ConstraintViolation(_)
            | DbError::RetryableTransaction(_)
            | DbError::Serialization(_)
            | DbError::QueryBuilder(_)
            | DbError::Transaction(_)
            | DbError::Database(_)
            | DbError::ConnectionError(_)
            | DbError::PoolError(_) => Self::Persistence(error),
        }
    }
}
impl From<MembershipError> for ApiError {
    fn from(error: MembershipError) -> Self {
        match error {
            MembershipError::MembershipNotFound => {
                ApiError::NotFound("Server membership not found".to_string())
            }
            MembershipError::MembershipRequired => {
                ApiError::Forbidden("Server membership is required for this operation".to_string())
            }
            MembershipError::AlreadyMember => {
                ApiError::Conflict("Member already has server membership".to_string())
            }

            MembershipError::OwnerCannotLeave => {
                ApiError::Forbidden("Server owner cannot leave the server".to_string())
            }
            MembershipError::Persistence(_) => ApiError::InternalError,
        }
    }
}
