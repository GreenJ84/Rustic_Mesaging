use diesel::Connection;

use crate::auth::session::SessionClock;
use crate::db::DbConn;
use crate::member::models::MemberId;
use crate::member::repository::MemberRepository;

use super::error::SystemRoleError;
use super::models::{
    BuiltInSystemRole, MemberSystemRole, NewSystemRole, RegistrationRoleKind, SystemRole,
    SystemRoleId,
};
use super::repository::SystemRoleRepository;

/// Business policy for the platform's single-role hierarchy.
pub(crate) struct SystemRoleService;

impl SystemRoleService {
    pub(crate) fn create_role(
        conn: &mut DbConn,
        requested_name: &str,
        authority_level: i32,
    ) -> Result<SystemRole, SystemRoleError> {
        let role = validate_custom_role(requested_name, authority_level)?;
        Ok(SystemRoleRepository::create(conn, role)?)
    }

    pub(crate) fn get_roles(conn: &mut DbConn) -> Result<Vec<SystemRole>, SystemRoleError> {
        Ok(SystemRoleRepository::read_all(conn)?)
    }

    pub(crate) fn update_role(
        conn: &mut DbConn,
        role_id: SystemRoleId,
        requested_name: &str,
        authority_level: i32,
    ) -> Result<SystemRole, SystemRoleError> {
        let changes = validate_custom_role(requested_name, authority_level)?;
        conn.transaction::<SystemRole, SystemRoleError, _>(|conn| {
            let existing = SystemRoleRepository::find_by_id_for_update(conn, role_id)?
                .ok_or(SystemRoleError::RoleNotFound)?;
            if existing.system_managed {
                return Err(SystemRoleError::CannotModifyBuiltInRole);
            }
            Ok(SystemRoleRepository::update_by_id(conn, role_id, changes)?)
        })
    }

    pub(crate) fn delete_role(
        conn: &mut DbConn,
        actor_id: MemberId,
        role_id: SystemRoleId,
    ) -> Result<(), SystemRoleError> {
        conn.transaction(|conn| {
            let role = SystemRoleRepository::find_by_id_for_update(conn, role_id)?
                .ok_or(SystemRoleError::RoleNotFound)?;
            if role.system_managed {
                return Err(SystemRoleError::CannotDeleteBuiltInRole);
            }

            let fallback = SystemRoleRepository::lock_next_lower_role(conn, role.authority_level)?
                .ok_or(SystemRoleError::ConfigurationFailure)?;

            SystemRoleRepository::reassign_role(
                conn,
                role.id,
                fallback.id,
                actor_id,
                SessionClock::now_naive(),
            )?;
            SystemRoleRepository::delete_by_id(conn, role.id)?;
            Ok(())
        })
    }

    pub(crate) fn issue_registration_role(
        conn: &mut DbConn,
        member_id: MemberId,
        kind: RegistrationRoleKind,
    ) -> Result<(), SystemRoleError> {
        let assignment_id = SystemRoleRepository::get_role_id(conn, kind.role().as_str())?
            .ok_or(SystemRoleError::ConfigurationFailure)?;
        SystemRoleRepository::create_assignment(
            conn,
            MemberSystemRole {
                member_id,
                role_id: assignment_id,
                assigned_by: None,
                assigned_at: SessionClock::now_naive(),
            },
        )?;
        Ok(())
    }

    pub(crate) fn get_member_role(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<SystemRole, SystemRoleError> {
        match SystemRoleRepository::get_member_role(conn, member_id)? {
            Some(role) => Ok(role),
            None => Err(SystemRoleError::AssignmentNotFound),
        }
    }

    pub(crate) fn attest_authorization(
        conn: &mut DbConn,
        member_id: MemberId,
        required_role: BuiltInSystemRole,
    ) -> Result<(), SystemRoleError> {
        let current_role = SystemRoleRepository::get_member_role(conn, member_id)?
            .ok_or(SystemRoleError::AssignmentNotFound)?;
        if current_role.authority_level < required_role.authority_level() {
            return Err(SystemRoleError::InsufficientPrivileges);
        }
        Ok(())
    }

    pub(crate) fn assign_role(
        conn: &mut DbConn,
        actor_id: MemberId,
        member_id: MemberId,
        new_role_id: SystemRoleId,
    ) -> Result<(), SystemRoleError> {
        conn.transaction(|conn| {
            if MemberRepository::find_by_id(conn, member_id)?.is_none() {
                return Err(SystemRoleError::MemberNotFound);
            }
            // Lock the member's current assignment, if any, to prevent concurrent modifications.
            let assignment_lock = SystemRoleRepository::lock_member_assignment(conn, member_id)?;
            // Lock the new role before its assignment.
            let new_role = SystemRoleRepository::find_by_id_for_update(conn, new_role_id)?
                .ok_or(SystemRoleError::RoleNotFound)?;

            let now = SessionClock::now_naive();
            match assignment_lock {
                None => {
                    SystemRoleRepository::create_assignment(
                        conn,
                        MemberSystemRole {
                            member_id,
                            role_id: new_role.id,
                            assigned_by: Some(actor_id),
                            assigned_at: now,
                        },
                    )?;
                }
                Some(current_role) => {
                    // If the member already has the requested role, no action is needed.
                    if current_role.role_id == new_role.id {
                        return Ok(());
                    }
                    // Get the current role of the assignment.
                    let current =
                        SystemRoleRepository::get_member_role(conn, current_role.member_id)?
                            .ok_or(SystemRoleError::ConfigurationFailure)?;
                    if current.authority_level == BuiltInSystemRole::Admin.authority_level() {
                        // Check if the current role is the last administrator before revoking it.
                        if SystemRoleRepository::last_admin_check(conn)? {
                            return Err(SystemRoleError::CannotRevokeLastAdministrator);
                        }
                    }
                    SystemRoleRepository::update_assignment(
                        conn,
                        member_id,
                        new_role.id,
                        actor_id,
                        now,
                    )?;
                }
            }
            Ok(())
        })
    }
}

fn validate_custom_role(
    requested_name: &str,
    authority_level: i32,
) -> Result<NewSystemRole, SystemRoleError> {
    let name = normalize_role_name(requested_name)?;
    if !(BuiltInSystemRole::Banned.authority_level() < authority_level
        && authority_level < BuiltInSystemRole::Admin.authority_level())
    {
        return Err(SystemRoleError::InvalidAuthorityLevel(format!(
            "must be between {} and {}",
            BuiltInSystemRole::Banned.authority_level(),
            BuiltInSystemRole::Admin.authority_level()
        )));
    }
    if BuiltInSystemRole::from_authority_level(authority_level).is_some()
        || BuiltInSystemRole::from_str(&name).is_some()
    {
        return Err(SystemRoleError::CannotModifyBuiltInRole);
    }
    Ok(NewSystemRole {
        name,
        authority_level,
    })
}

fn normalize_role_name(value: &str) -> Result<String, SystemRoleError> {
    let value = value.trim().to_ascii_lowercase();
    if !(3..=32).contains(&value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_".contains(&byte))
    {
        return Err(SystemRoleError::InvalidRoleName);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{normalize_role_name, validate_custom_role};
    use crate::auth::system_roles::SystemRoleError;

    #[test]
    fn normalizes_valid_role_names() {
        assert_eq!(
            normalize_role_name("  Content-Mod_2 ").unwrap(),
            "content-mod_2"
        );
    }

    #[test]
    fn rejects_invalid_role_names() {
        for invalid in ["ab", "contains space", "admin!", "åäö"] {
            assert!(normalize_role_name(invalid).is_err());
        }
    }

    #[test]
    fn reserves_built_in_names_and_authorities() {
        assert!(matches!(
            validate_custom_role("admin", 45),
            Err(SystemRoleError::CannotModifyBuiltInRole)
        ));
        assert!(matches!(
            validate_custom_role("helper", 40),
            Err(SystemRoleError::CannotModifyBuiltInRole)
        ));
    }

    #[test]
    fn validates_custom_authority_range() {
        assert!(matches!(
            validate_custom_role("helper", 51),
            Err(SystemRoleError::InvalidAuthorityLevel(_))
        ));
        assert!(validate_custom_role("helper", 35).is_ok());
    }
}
