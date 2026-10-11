use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{
    password_hash::Error as PasswordHashError, Argon2, PasswordHash, PasswordHasher,
    PasswordVerifier,
};

use crate::auth::models::RegisterInput;

use super::error::{AuthError, PasswordViolation, MAX_PASSWORD_LENGTH, MIN_PASSWORD_LENGTH};

pub struct PasswordManager;
impl PasswordManager {
    pub fn validate(input: &RegisterInput) -> Result<String, AuthError> {
        let password: &String = &input.password;

        let common_passwords = vec!["password", "123456", "qwerty", "letmein"];
        if common_passwords.contains(&password.as_str()) {
            return Err(AuthError::PasswordRejected(
                PasswordViolation::CommonPassword,
            ));
        }
        if password.contains(&input.username) || password.contains(&input.email) {
            return Err(AuthError::PasswordRejected(
                PasswordViolation::ContainsIdentity,
            ));
        }
        Ok(Self::hash(password)?)
    }

    fn hash(password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        match Argon2::default()
            .hash_password(password.as_bytes(), &salt){
                Ok(hash) => Ok(hash.to_string()),
                Err(e) => {
                    log::error!("Failed to hash password: {}", e);
                    Err(AuthError::HashingError)
                }
            }
    }

    pub fn verify(password: &str, hash: &str) -> Result<(), AuthError> {
        let parsed = PasswordHash::new(hash).map_err(|_| AuthError::HashingError)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|er| AuthError::InvalidCredentials)
    }
}
