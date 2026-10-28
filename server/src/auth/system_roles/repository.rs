use diesel::prelude::*;

use crate::auth::system_roles::BuiltInSystemRole;
use crate::infra::{DbConn, DbError};
use crate::member::models::MemberId;
use crate::schema::{member_system_role::dsl as assignments, system_role::dsl as roles};

use super::models::{MemberSystemRole, NewSystemRole, SystemRole, SystemRoleId};

/// Stateless persistence operations for system roles and their single-role
/// member assignments. Transaction policy belongs to `SystemRoleService`.
pub(super) struct SystemRoleRepository;

// System Role Operations ==========================
impl SystemRoleRepository {
    // Create ==========================
    /// Creates a new system role in the database.
    pub(super) fn create(conn: &mut DbConn, role: NewSystemRole) -> Result<SystemRole, DbError> {
        diesel::insert_into(roles::system_role)
            .values(role)
            .returning(SystemRole::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Read ==========================
    /// Finds a system role by its ID, returning `None` if not found. Locks the row for update if found.
    pub(super) fn find_by_id_for_update(
        conn: &mut DbConn,
        role_id: SystemRoleId,
    ) -> Result<Option<SystemRole>, DbError> {
        roles::system_role
            .find(role_id)
            .for_update()
            .select(SystemRole::as_select())
            .first(conn)
            .optional()
            .map_err(DbError::from)
    }

    pub(super) fn get_role_id(
        conn: &mut DbConn,
        role_name: &str,
    ) -> Result<Option<SystemRoleId>, DbError> {
        roles::system_role
            .filter(roles::name.eq(role_name))
            .select(roles::id)
            .first(conn)
            .optional()
            .map_err(DbError::from)
    }

    /// Finds all system roles, ordered by authority level descending and name ascending.
    pub(super) fn read_all(conn: &mut DbConn) -> Result<Vec<SystemRole>, DbError> {
        roles::system_role
            .order((roles::authority_level.desc(), roles::name.asc()))
            .select(SystemRole::as_select())
            .load(conn)
            .map_err(DbError::from)
    }

    // Update ==========================
    /// Updates a system role by its ID, returning the updated role. Returns an error if the role does not exist.
    pub(super) fn update_by_id(
        conn: &mut DbConn,
        role_id: SystemRoleId,
        changes: NewSystemRole,
    ) -> Result<SystemRole, DbError> {
        diesel::update(roles::system_role.find(role_id))
            .set(changes)
            .returning(SystemRole::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Delete ==========================
    /// Locks the next lower system role by authority level.
    pub(super) fn lock_next_lower_role(
        conn: &mut DbConn,
        authority_level: i32,
    ) -> Result<Option<SystemRole>, DbError> {
        roles::system_role
            .filter(roles::authority_level.lt(authority_level))
            .order(roles::authority_level.desc())
            .for_update()
            .select(SystemRole::as_select())
            .first(conn)
            .optional()
            .map_err(DbError::from)
    }

    /// Deletes a system role by its ID. Returns an error if the role does not exist.
    pub(super) fn delete_by_id(conn: &mut DbConn, role_id: SystemRoleId) -> Result<(), DbError> {
        let deleted = diesel::delete(roles::system_role.find(role_id))
            .execute(conn)
            .map_err(DbError::from)?;
        if deleted == 1 {
            Ok(())
        } else {
            Err(DbError::NotFound)
        }
    }
}

// Member Role Assignment Operations ==========================
impl SystemRoleRepository {
    // Create ==========================
    /// Inserts a new member system role assignment into the database. New Member Role initialization.
    pub(super) fn create_assignment(
        conn: &mut DbConn,
        assignment: MemberSystemRole,
    ) -> Result<MemberSystemRole, DbError> {
        diesel::insert_into(assignments::member_system_role)
            .values(assignment)
            .returning(MemberSystemRole::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    // Read ==========================
    /// Retrieves the system role assigned to a specific member, if any.
    pub(super) fn get_member_role(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<Option<SystemRole>, DbError> {
        assignments::member_system_role
            .filter(assignments::member_id.eq(member_id))
            .inner_join(roles::system_role.on(roles::id.eq(assignments::role_id)))
            .select(SystemRole::as_select())
            .first(conn)
            .optional()
            .map_err(DbError::from)
    }

    // Update ==========================
    /// Locks the member's role assignment for update, returning the assignment if it exists.
    pub(super) fn lock_member_assignment(
        conn: &mut DbConn,
        member_id: MemberId,
    ) -> Result<Option<MemberSystemRole>, DbError> {
        assignments::member_system_role
            .filter(assignments::member_id.eq(member_id))
            .for_update()
            .select(MemberSystemRole::as_select())
            .first(conn)
            .optional()
            .map_err(DbError::from)
    }

    /// Updates the role assignment for a specific member. This is used when changing a member's role.
    pub(super) fn update_assignment(
        conn: &mut DbConn,
        member_id: MemberId,
        new_role_id: SystemRoleId,
        assigned_by: MemberId,
        assigned_at: chrono::NaiveDateTime,
    ) -> Result<MemberSystemRole, DbError> {
        diesel::update(assignments::member_system_role.filter(assignments::member_id.eq(member_id)))
            .set((
                assignments::role_id.eq(new_role_id),
                assignments::assigned_by.eq(Some(assigned_by)),
                assignments::assigned_at.eq(assigned_at),
            ))
            .returning(MemberSystemRole::as_returning())
            .get_result(conn)
            .map_err(DbError::from)
    }

    /// Checks if the last administrator is being revoked.
    pub(super) fn last_admin_check(conn: &mut DbConn) -> Result<bool, DbError> {
        let admin_role_id = roles::system_role
            .filter(roles::authority_level.eq(BuiltInSystemRole::Admin.authority_level()))
            .select(roles::id)
            .for_update()
            .first::<SystemRoleId>(conn)?;

        let admin_count = assignments::member_system_role
            .filter(assignments::role_id.eq(admin_role_id))
            .count()
            .get_result::<i64>(conn)?;

        Ok(admin_count <= 1)
    }

    /// Reassigns all members from one role to another. This is used when a role is being deleted and its members need to be reassigned to a different role.
    pub(super) fn reassign_role(
        conn: &mut DbConn,
        old_role_id: SystemRoleId,
        new_role_id: SystemRoleId,
        assigned_by: MemberId,
        assigned_at: chrono::NaiveDateTime,
    ) -> Result<usize, DbError> {
        diesel::update(assignments::member_system_role.filter(assignments::role_id.eq(old_role_id)))
            .set((
                assignments::role_id.eq(new_role_id),
                assignments::assigned_by.eq(Some(assigned_by)),
                assignments::assigned_at.eq(assigned_at),
            ))
            .execute(conn)
            .map_err(DbError::from)
    }
}
