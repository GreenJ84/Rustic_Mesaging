use crate::{
    auth::system_roles::SystemRoleError,
    db::{DbError, DieselError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberError {
    NotFound,

    UsernameTaken,
    EmailTaken,
    InvalidChangeset(String),
    InvalidSearchTerm(String),

    SystemRoleError(SystemRoleError),
    CannotDeleteLastAdministrator,
    DeletionBlocked,

    PersistenceFailure,
}
impl From<DbError> for MemberError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => Self::NotFound,

            DbError::Conflict(DieselError::DatabaseError(_, information)) => {
                match information.constraint_name() {
                    Some("member_username_key") => Self::UsernameTaken,
                    Some("member_email_key") => Self::EmailTaken,
                    _ => Self::PersistenceFailure,
                }
            }

            _ => Self::PersistenceFailure,
        }
    }
}
impl From<SystemRoleError> for MemberError {
    fn from(error: SystemRoleError) -> Self {
        Self::SystemRoleError(error)
    }
}
impl From<DieselError> for MemberError {
    fn from(error: DieselError) -> Self {
        DbError::from(error).into()
    }
}
