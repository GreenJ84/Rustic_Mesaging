use rocket::http::Status;

use crate::infra::{DbError, DieselError};

#[derive(Debug)]
pub(crate) enum AccountDeletionError {
    LastAdministrator,
    ServerOwner,
    ChatOwner,
    UnderInvestigation,
    Persistence(DbError),
}

impl AccountDeletionError {
    pub(crate) fn message(&self) -> &'static str {
        match self {
            Self::LastAdministrator => "Cannot delete the last administrator",
            Self::ServerOwner => "Transfer or delete owned servers before deleting the account",
            Self::ChatOwner => "Delete owned chats before deleting the account",
            Self::UnderInvestigation => "Cannot delete a member under investigation",
            Self::Persistence(_) => "Account deletion failed",
        }
    }

    pub(crate) fn status(&self) -> Status {
        match self {
            Self::LastAdministrator
            | Self::ServerOwner
            | Self::ChatOwner
            | Self::UnderInvestigation => Status::Conflict,
            Self::Persistence(_) => Status::InternalServerError,
        }
    }
}

impl From<DbError> for AccountDeletionError {
    fn from(error: DbError) -> Self {
        if let DbError::ConstraintViolation(DieselError::DatabaseError(_, information)) = &error {
            return match information.constraint_name() {
                Some("server_owner_id_fkey") => Self::ServerOwner,
                Some("chat_owner_member_id_fkey") => Self::ChatOwner,
                _ => Self::Persistence(error),
            };
        }
        Self::Persistence(error)
    }
}
