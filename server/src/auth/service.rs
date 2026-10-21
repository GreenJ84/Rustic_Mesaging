use diesel::Connection;

use crate::auth::system_roles::SystemRoleId;
use crate::database::{Database, DbConn, DbError, DieselError};
use crate::id_generator::IdGenerator;
use crate::member::models::{MemberId, MemberProfile};
use crate::member::repository::MemberRepository;

use super::error::AuthError;
use super::models::{AuthOutcome, ChangePasswordInput, LoginInput, RegisterInput};
use super::password::PasswordManager;
use super::session::{AuthenticatedPrincipal, SessionConfig, SessionMetadata, SessionService};
use super::system_roles::{
    BuiltInSystemRole, RegistrationRoleKind, SystemRole, SystemRoleError, SystemRoleService,
};

/// Top-level authentication service. It owns connection acquisition and
/// coordinates member, role, and session work in one transaction where the
/// complete use case must be atomic.
pub struct AuthService {
    database: Database,
    id_generator: IdGenerator,
    sessions: SessionService,
}

impl AuthService {
    pub fn new(
        database: Database,
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
            database,
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
        let mut conn = self.database.get_connection()?;
        let session_id = self.id_generator.generate_uuid();
        let family_id = self.id_generator.generate_uuid();

        conn.transaction(|conn| {
            let new_member = input.validate(conn)?;
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
        let mut conn = self.database.get_connection()?;
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
        let mut conn = self.database.get_connection()?;
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
        let mut conn = self.database.get_connection()?;
        self.sessions
            .authenticate_access_token(&mut conn, access_token)
            .map_err(AuthError::from)
    }

    pub(crate) fn authorize(
        &self,
        member_id: MemberId,
        minimum: BuiltInSystemRole,
    ) -> Result<(), AuthError> {
        let mut conn = self.database.get_connection()?;
        match SystemRoleService::attest_authorization(&mut conn, member_id, minimum) {
            Ok(()) => Ok(()),
            Err(SystemRoleError::InsufficientPrivileges) => Err(AuthError::InsufficientPrivileges),
            Err(error) => Err(map_role_auth_error(error)),
        }
    }

    pub(crate) fn is_administrator(&self, member_id: MemberId) -> Result<bool, AuthError> {
        let mut conn = self.database.get_connection()?;
        let role = SystemRoleService::get_member_role(&mut conn, member_id)
            .map_err(map_role_auth_error)?;
        Ok(role.authority_level >= BuiltInSystemRole::Admin.authority_level())
    }

    pub(crate) fn logout(&self, refresh_token: &str) -> Result<(), AuthError> {
        let mut conn = self.database.get_connection()?;
        self.sessions.logout(&mut conn, refresh_token)?;
        Ok(())
    }

    pub(crate) fn logout_all(&self, member_id: MemberId) -> Result<(), AuthError> {
        let mut conn = self.database.get_connection()?;
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

        let mut conn = self.database.get_connection()?;
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
        let mut conn = self.database.get_connection()?;
        Self::require_role(&mut conn, principal.member_id, BuiltInSystemRole::User)?;
        MemberRepository::find_by_id(&mut conn, principal.member_id)?
            .map(|member| member.into_profile())
            .ok_or(AuthError::AccountNotFound)
    }

    pub(crate) fn secure_cookies(&self) -> bool {
        self.sessions.secure_cookies()
    }

    fn require_role(
        conn: &mut DbConn,
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

impl AuthService {
    pub(crate) fn create_role(
        &self,
        requested_name: &str,
        authority_level: i32,
    ) -> Result<SystemRole, SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        Ok(SystemRoleService::create_role(
            &mut conn,
            requested_name,
            authority_level,
        )?)
    }

    pub(crate) fn get_roles(&self) -> Result<Vec<SystemRole>, SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        Ok(SystemRoleService::get_roles(&mut conn)?)
    }

    pub(crate) fn update_role(
        &self,
        role_id: SystemRoleId,
        requested_name: &str,
        authority_level: i32,
    ) -> Result<SystemRole, SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        Ok(SystemRoleService::update_role(
            &mut conn,
            role_id,
            requested_name,
            authority_level,
        )?)
    }

    pub(crate) fn delete_role(
        &self,
        actor_id: MemberId,
        role_id: SystemRoleId,
    ) -> Result<(), SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        SystemRoleService::delete_role(&mut conn, actor_id, role_id)
    }
}
// Role Assignment Operations ==========================
impl AuthService {
    pub(crate) fn issue_registration_role(
        &self,
        member_id: MemberId,
        kind: RegistrationRoleKind,
    ) -> Result<(), SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        SystemRoleService::issue_registration_role(&mut conn, member_id, kind)
    }

    pub(crate) fn get_member_role(
        &self,
        member_id: MemberId,
    ) -> Result<SystemRole, SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        SystemRoleService::get_member_role(&mut conn, member_id)
    }

    pub(crate) fn attest_authorization(
        &self,
        member_id: MemberId,
        required_role: BuiltInSystemRole,
    ) -> Result<(), SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        SystemRoleService::attest_authorization(&mut conn, member_id, required_role)
    }

    pub(crate) fn assign_role(
        &self,
        actor_id: MemberId,
        member_id: MemberId,
        new_role_id: SystemRoleId,
    ) -> Result<(), SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        SystemRoleService::assign_role(&mut conn, actor_id, member_id, new_role_id)
    }

    pub(crate) fn reset_role(
        &self,
        actor_id: MemberId,
        member_id: MemberId,
    ) -> Result<(), SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        SystemRoleService::reset_role(&mut conn, actor_id, member_id)
    }

    pub(crate) fn validate_member_deletion(
        &self,
        member_id: MemberId,
    ) -> Result<bool, SystemRoleError> {
        let mut conn = self.database.get_connection()?;
        SystemRoleService::validate_member_deletion(&mut conn, member_id)
    }
}
