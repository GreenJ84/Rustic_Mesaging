use diesel::Connection;

use crate::db::{Db, DbError, DbPool, DieselError};
use crate::id_generator::IdGenerator;
use crate::member::models::{MemberId, MemberProfile};
use crate::member::repository::MemberRepository;

use super::error::AuthError;
use super::models::{AuthOutcome, ChangePasswordInput, LoginInput, RegisterInput};
use super::password::PasswordManager;
use super::session::{AuthenticatedPrincipal, SessionConfig, SessionMetadata, SessionService};
use super::system_roles::{
    BuiltInSystemRole, RegistrationRoleKind, SystemRoleError, SystemRoleService,
};

/// Top-level authentication service. It owns connection acquisition and
/// coordinates member, role, and session work in one transaction where the
/// complete use case must be atomic.
pub struct AuthService {
    pool: DbPool,
    id_generator: IdGenerator,
    sessions: SessionService,
}

impl AuthService {
    pub fn new(
        pool: DbPool,
        id_generator: IdGenerator,
        jwt_secret: &[u8],
        cookie_secure: bool,
    ) -> Self {
        let sessions = SessionService::new(
            jwt_secret,
            SessionConfig {
                issuer: "rustic-messaging".to_string(),
                audience: "rustic-messaging-users".to_string(),
                access_ttl: chrono::Duration::minutes(15),
                refresh_ttl: chrono::Duration::days(7),
                cookie_secure,
            },
        );
        Self {
            pool,
            id_generator,
            sessions,
        }
    }

    pub(crate) fn register(
        &self,
        input: RegisterInput,
        registration_role: RegistrationRoleKind,
        metadata: SessionMetadata,
    ) -> Result<AuthOutcome, AuthError> {
        let mut conn = Db::connect(&self.pool)?;
        let session_id = self.id_generator.generate_uuid();
        let family_id = self.id_generator.generate_uuid();

        conn.transaction(|conn| {
            let mut new_member = input.validate(conn)?;
            let member =
                MemberRepository::create(conn, new_member).map_err(map_registration_write_error)?;
            SystemRoleService::issue_registration_role(conn, member.id(), registration_role)
                .map_err(map_role_auth_error)?;
            let credentials =
                self.sessions
                    .issue(conn, session_id, family_id, member.id(), metadata)?;
            Ok(AuthOutcome {
                member: member.into_profile(),
                credentials,
            })
        })
    }

    pub(crate) fn login(
        &self,
        input: LoginInput,
        metadata: SessionMetadata,
    ) -> Result<AuthOutcome, AuthError> {
        let mut conn = Db::connect(&self.pool)?;
        let member = MemberRepository::find_by_username(&mut conn, &input.username)?
            .ok_or(AuthError::InvalidCredentials)?;
        PasswordManager::verify(&input.password, member.password_hash())?;
        Self::require_role(&mut conn, member.id(), BuiltInSystemRole::User)?;

        let credentials = self.sessions.issue(
            &mut conn,
            self.id_generator.generate_uuid(),
            self.id_generator.generate_uuid(),
            member.id(),
            metadata,
        )?;
        Ok(AuthOutcome {
            member: member.into_profile(),
            credentials,
        })
    }

    pub(crate) fn refresh(
        &self,
        member_id: MemberId,
        refresh_token: String,
        metadata: SessionMetadata,
    ) -> Result<AuthOutcome, AuthError> {
        let mut conn = Db::connect(&self.pool)?;
        let member = MemberRepository::find_by_id(&mut conn, member_id)?
            .ok_or(AuthError::InvalidCredentials)?;
        Self::require_role(&mut conn, member.id(), BuiltInSystemRole::User)?;
        let credentials = self.sessions.rotate(
            &mut conn,
            member_id,
            refresh_token,
            self.id_generator.generate_uuid(),
            metadata,
        )?;

        Ok(AuthOutcome {
            member: member.into_profile(),
            credentials,
        })
    }

    pub(crate) fn authenticate(
        &self,
        access_token: &str,
    ) -> Result<AuthenticatedPrincipal, AuthError> {
        let mut conn = Db::connect(&self.pool)?;
        self.sessions
            .authenticate_access_token(&mut conn, access_token)
            .map_err(AuthError::from)
    }

    pub(crate) fn authorize(
        &self,
        member_id: MemberId,
        minimum: BuiltInSystemRole,
    ) -> Result<(), AuthError> {
        let mut conn = Db::connect(&self.pool)?;
        match SystemRoleService::attest_authorization(&mut conn, member_id, minimum) {
            Ok(()) => Ok(()),
            Err(SystemRoleError::InsufficientPrivileges) => Err(AuthError::InsufficientPrivileges),
            Err(error) => Err(map_role_auth_error(error)),
        }
    }

    pub(crate) fn is_administrator(&self, member_id: MemberId) -> Result<bool, AuthError> {
        let mut conn = Db::connect(&self.pool)?;
        let role = SystemRoleService::get_member_role(&mut conn, member_id)
            .map_err(map_role_auth_error)?;
        Ok(role.authority_level >= BuiltInSystemRole::Admin.authority_level())
    }

    pub(crate) fn logout(&self, refresh_token: &str) -> Result<(), AuthError> {
        let mut conn = Db::connect(&self.pool)?;
        self.sessions.logout(&mut conn, refresh_token)?;
        Ok(())
    }

    pub(crate) fn logout_all(&self, member_id: MemberId) -> Result<(), AuthError> {
        let mut conn = Db::connect(&self.pool)?;
        self.sessions.logout_all(&mut conn, member_id)?;
        Ok(())
    }

    pub(crate) fn change_password(
        &self,
        principal: &AuthenticatedPrincipal,
        input: ChangePasswordInput,
    ) -> Result<(), AuthError> {
        if input.new_password != input.confirm_password {
            return Err(AuthError::PasswordConfirmationMismatch);
        }

        let mut conn = Db::connect(&self.pool)?;
        let member = MemberRepository::find_by_id(&mut conn, principal.member_id)?
            .ok_or(AuthError::AccountNotFound)?;
        PasswordManager::verify(&input.current_password, member.password_hash())?;
        PasswordManager::validate_candidate(
            &input.new_password,
            member.username(),
            member.email(),
        )?;
        let password_hash = PasswordManager::hash(&input.new_password)?;

        conn.transaction(|conn| {
            if !MemberRepository::update_password(
                conn,
                principal.member_id,
                member.password_hash(),
                password_hash,
            )? {
                return Err(AuthError::InvalidCredentials);
            }
            self.sessions.logout_all(conn, principal.member_id)?;
            Ok(())
        })
    }

    pub(crate) fn current_member(
        &self,
        principal: &AuthenticatedPrincipal,
    ) -> Result<MemberProfile, AuthError> {
        let mut conn = Db::connect(&self.pool)?;
        Self::require_role(&mut conn, principal.member_id, BuiltInSystemRole::User)?;
        MemberRepository::find_by_id(&mut conn, principal.member_id)?
            .map(|member| member.into_profile())
            .ok_or(AuthError::AccountNotFound)
    }

    pub(crate) fn secure_cookies(&self) -> bool {
        self.sessions.secure_cookies()
    }

    fn require_role(
        conn: &mut crate::db::DbConn,
        member_id: MemberId,
        minimum: BuiltInSystemRole,
    ) -> Result<(), AuthError> {
        match SystemRoleService::attest_authorization(conn, member_id, minimum) {
            Ok(()) => Ok(()),
            Err(error) => Err(map_role_auth_error(error)),
        }
    }
}

fn map_role_auth_error(error: SystemRoleError) -> AuthError {
    match error {
        SystemRoleError::InsufficientPrivileges => AuthError::InsufficientPrivileges,
        SystemRoleError::MemberNotFound => AuthError::AccountNotFound,
        SystemRoleError::AssignmentNotFound => AuthError::AccountDisabled,
        _ => AuthError::PersistenceFailure,
    }
}

fn map_registration_write_error(error: DbError) -> AuthError {
    if let DbError::Conflict(DieselError::DatabaseError(_, information)) = &error {
        return match information.constraint_name() {
            Some("member_username_key") => AuthError::UsernameTaken,
            Some("member_email_key") => AuthError::EmailTaken,
            _ => AuthError::PersistenceFailure,
        };
    }
    AuthError::PersistenceFailure
}
