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
                    Some("friend_member_id_fkey")
                    | Some("friend_friend_id_fkey")
                    | Some("friend_request_sender_id_fkey")
                    | Some("friend_request_receiver_id_fkey") => SocialError::MemberNotFound,

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
            SocialError::Persistence(error) => {
                error.log_error("social domain");
                ApiError::InternalError
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SocialError;
    use crate::api::ApiError;
    use crate::infra::DbError;
    use diesel::result::{DatabaseErrorInformation, DatabaseErrorKind, Error as DieselError};
    use rocket::http::Status;

    struct ConstraintInformation(&'static str);

    impl DatabaseErrorInformation for ConstraintInformation {
        fn message(&self) -> &str {
            "test constraint violation"
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

    fn constraint(name: &'static str) -> DbError {
        DbError::ConstraintViolation(DieselError::DatabaseError(
            DatabaseErrorKind::CheckViolation,
            Box::new(ConstraintInformation(name)),
        ))
    }

    #[test]
    fn friendship_constraints_map_to_domain_conflicts() {
        assert!(matches!(
            SocialError::from(conflict("friend_pkey")),
            SocialError::AlreadyFriends
        ));
        assert!(matches!(
            SocialError::from(conflict("friend_request_already_friend")),
            SocialError::AlreadyFriends
        ));
    }

    #[test]
    fn request_constraints_map_to_duplicate_request_errors() {
        for name in ["friend_request_pkey", "friend_request_pair_unique"] {
            assert!(matches!(
                SocialError::from(conflict(name)),
                SocialError::RequestAlreadyExists
            ));
        }
    }

    #[test]
    fn check_constraints_map_to_input_errors() {
        for name in [
            "friend_members_must_differ",
            "friend_request_members_must_differ",
        ] {
            assert!(matches!(
                SocialError::from(constraint(name)),
                SocialError::SelfFriendship
            ));
        }
        assert!(matches!(
            SocialError::from(constraint("friend_ids_are_canonical")),
            SocialError::InvalidInput(_)
        ));
    }

    #[test]
    fn social_foreign_keys_map_to_member_not_found() {
        for name in [
            "friend_member_id_fkey",
            "friend_friend_id_fkey",
            "friend_request_sender_id_fkey",
            "friend_request_receiver_id_fkey",
        ] {
            assert!(matches!(
                SocialError::from(constraint(name)),
                SocialError::MemberNotFound
            ));
        }
    }

    #[test]
    fn unknown_database_constraints_remain_internal() {
        assert!(matches!(
            SocialError::from(conflict("unknown_constraint")),
            SocialError::Persistence(_)
        ));
        assert!(matches!(
            SocialError::from(DbError::ConnectionError("offline".into())),
            SocialError::Persistence(_)
        ));
    }

    #[test]
    fn social_errors_map_to_expected_http_status_classes() {
        let cases = [
            (
                SocialError::InvalidInput("invalid".into()),
                Status::BadRequest,
            ),
            (SocialError::SelfFriendship, Status::BadRequest),
            (SocialError::MemberNotFound, Status::NotFound),
            (SocialError::FriendshipNotFound, Status::NotFound),
            (SocialError::RequestNotFound, Status::NotFound),
            (SocialError::AlreadyFriends, Status::Conflict),
            (SocialError::RequestAlreadyExists, Status::Conflict),
            (SocialError::NotRequestRecipient, Status::Forbidden),
            (SocialError::NotRequestSender, Status::Forbidden),
            (
                SocialError::Persistence(DbError::ConnectionError("offline".into())),
                Status::InternalServerError,
            ),
        ];

        for (error, status) in cases {
            assert_eq!(ApiError::from(error).status(), status);
        }
    }
}
