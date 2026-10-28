use crate::api::ApiError;
use crate::database::{DbError, DieselError};
use crate::member::error::MemberError;

use super::membership::error::MembershipError;

/// Failures produced by server-entity validation, authorization, and persistence.
#[derive(Debug)]
pub(crate) enum ServerError {
    InvalidInput(String),
    InvalidChangeset(String),
    InvalidSearchTerm(String),

    NotFound,
    NameTaken,

    NotOwner,
    MissingAuthority,

    Membership(MembershipError),
    Member(MemberError),
    Persistence(DbError),
}

impl From<DbError> for ServerError {
    fn from(error: DbError) -> Self {
        match &error {
            DbError::NotFound => Self::NotFound,

            DbError::Conflict(DieselError::DatabaseError(_, information)) => {
                match information.constraint_name() {
                    Some("server_name_key") => Self::NameTaken,
                    Some("server_membership_pkey") => {
                        ServerError::Membership(MembershipError::AlreadyMember)
                    }
                    _ => Self::Persistence(error),
                }
            }

            DbError::ConstraintViolation(DieselError::DatabaseError(_, information)) => {
                match information.constraint_name() {
                    Some("server_membership_server_id_fkey") => Self::NotFound,
                    Some("server_membership_member_id_fkey") => Self::Member(MemberError::NotFound),
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
impl From<DieselError> for ServerError {
    fn from(error: DieselError) -> Self {
        Self::from(DbError::from(error))
    }
}

impl From<MembershipError> for ServerError {
    fn from(error: MembershipError) -> Self {
        Self::Membership(error)
    }
}

impl From<ServerError> for ApiError {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError::InvalidInput(message)
            | ServerError::InvalidChangeset(message)
            | ServerError::InvalidSearchTerm(message) => ApiError::BadRequest(message),

            ServerError::NotFound => ApiError::NotFound("Server not found".into()),

            ServerError::NameTaken => {
                ApiError::Conflict("A server with that name already exists".into())
            }

            ServerError::NotOwner => {
                ApiError::Forbidden("Server ownership is required for this operation".into())
            }
            ServerError::MissingAuthority => ApiError::Forbidden(
                "You do not have the required authority for this operation".into(),
            ),

            ServerError::Membership(error) => ApiError::from(error),
            ServerError::Member(error) => ApiError::from(error),
            ServerError::Persistence(_) => ApiError::InternalError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ServerError;
    use crate::api::ApiError;
    use crate::database::{DbError, DieselError};
    use rocket::http::Status;

    #[test]
    fn diesel_not_found_maps_to_not_found() {
        assert!(matches!(
            ServerError::from(DieselError::NotFound),
            ServerError::NotFound
        ));
    }

    #[test]
    fn database_not_found_maps_to_not_found() {
        assert!(matches!(
            ServerError::from(DbError::NotFound),
            ServerError::NotFound
        ));
    }

    #[test]
    fn authorization_failures_map_to_forbidden() {
        assert_eq!(
            ApiError::from(ServerError::NotOwner).status(),
            Status::Forbidden
        );
    }
}
