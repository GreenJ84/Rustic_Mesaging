use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{
    password_hash::Error as PasswordHashError, Argon2, PasswordHash, PasswordHasher,
    PasswordVerifier,
};

use crate::member::models::{ChangePasswordInput, NewMemberInput};

/// Password violation rules
pub const MIN_PASSWORD_LENGTH: usize = 15;
pub const MAX_PASSWORD_LENGTH: usize = 128;
#[derive(Debug)]
pub(crate) enum PasswordError {
    InvalidLength,
    CommonPassword,
    ContainsIdentity,
    HashingError,
    PasswordMismatch,
}
impl PasswordError {
    pub fn message(&self) -> String {
        match self {
            Self::InvalidLength => format!(
                "Password must be between {MIN_PASSWORD_LENGTH} and {MAX_PASSWORD_LENGTH} characters long."
            ),
            Self::CommonPassword => "Password is too common.".to_string(),
            Self::ContainsIdentity => {
                "Password must not contain your username or email.".to_string()
            }
            Self::HashingError => "An error occurred while hashing the password.".to_string(),
            Self::PasswordMismatch => "Passwords do not match.".to_string(),
        }
    }
}
impl From<PasswordHashError> for PasswordError {
    fn from(e: PasswordHashError) -> Self {
        log::error!("Password hashing error: {}", e);
        PasswordError::HashingError
    }
}

pub struct PasswordManager;
impl PasswordManager {
    pub fn validate(input: &NewMemberInput) -> Result<String, PasswordError> {
        Self::validate_candidate(&input.password, &input.username, &input.email)?;
        Self::hash(&input.password)
    }

    pub(crate) fn validate_candidate(
        password: &str,
        username: &str,
        email: &str,
    ) -> Result<(), PasswordError> {
        if !(MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH).contains(&password.chars().count()) {
            return Err(PasswordError::InvalidLength);
        }

        let common_passwords = ["password", "123456", "qwerty", "letmein"];
        if common_passwords.contains(&password) {
            return Err(PasswordError::CommonPassword);
        }
        let normalized = password.to_ascii_lowercase();
        if normalized.contains(&username.to_ascii_lowercase())
            || normalized.contains(&email.to_ascii_lowercase())
        {
            return Err(PasswordError::ContainsIdentity);
        }
        Ok(())
    }

    pub(crate) fn hash(password: &str) -> Result<String, PasswordError> {
        let salt = SaltString::generate(&mut OsRng);
        match Argon2::default().hash_password(password.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => {
                log::error!("Failed to hash password: {}", e);
                Err(PasswordError::HashingError)
            }
        }
    }

    pub fn verify(password: &str, hash: &str) -> Result<(), PasswordError> {
        let parsed = PasswordHash::new(hash).map_err(|_| PasswordError::HashingError)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| PasswordError::PasswordMismatch)
    }
}
