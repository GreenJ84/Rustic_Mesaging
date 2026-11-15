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

            // Server membership currently has one unique constraint: its
            // composite primary key. Preserve the domain conflict when the
            // database driver omits constraint metadata.
            DbError::Conflict(DieselError::DatabaseError(_, _)) => Self::AlreadyMember,

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
            MembershipError::Persistence(error) => {
                error.log_error("server membership");
                ApiError::InternalError
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MembershipError;
    use crate::api::ApiError;
    use crate::infra::DbError;
    use diesel::result::{DatabaseErrorInformation, DatabaseErrorKind, Error as DieselError};
    use rocket::http::Status;

    struct ConstraintInformation(&'static str);

    impl DatabaseErrorInformation for ConstraintInformation {
        fn message(&self) -> &str {
            "test database error"
        }
        fn details(&self) -> Option<&str> {
            None
        }
        fn hint(&self) -> Option<&str> {
            None
        }
        fn table_name(&self) -> Option<&str> {
            None
        }
        fn column_name(&self) -> Option<&str> {
            None
        }
        fn constraint_name(&self) -> Option<&str> {
            Some(self.0)
        }
        fn statement_position(&self) -> Option<i32> {
            None
        }
    }

    fn conflict(name: &'static str) -> DbError {
        DbError::Conflict(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            Box::new(ConstraintInformation(name)),
        ))
    }

    #[test]
    fn membership_primary_key_maps_to_already_member() {
        assert!(matches!(
            MembershipError::from(conflict("server_membership_pkey")),
            MembershipError::AlreadyMember
        ));
    }

    #[test]
    fn missing_rows_map_to_missing_membership() {
        assert!(matches!(
            MembershipError::from(DbError::NotFound),
            MembershipError::MembershipNotFound
        ));
    }

    #[test]
    fn unrecognized_unique_conflicts_remain_membership_conflicts() {
        assert!(matches!(
            MembershipError::from(conflict("unknown_constraint")),
            MembershipError::AlreadyMember
        ));
    }

    #[test]
    fn non_constraint_database_errors_remain_internal() {
        assert!(matches!(
            MembershipError::from(DbError::ConnectionError("offline".into())),
            MembershipError::Persistence(_)
        ));
    }

    #[test]
    fn membership_errors_map_to_expected_http_status_classes() {
        let cases = [
            (MembershipError::MembershipNotFound, Status::NotFound),
            (MembershipError::MembershipRequired, Status::Forbidden),
            (MembershipError::AlreadyMember, Status::Conflict),
            (MembershipError::OwnerCannotLeave, Status::Forbidden),
            (
                MembershipError::Persistence(DbError::ConnectionError("offline".into())),
                Status::InternalServerError,
            ),
        ];

        for (error, status) in cases {
            assert_eq!(ApiError::from(error).status(), status);
        }
    }
}
