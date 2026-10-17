use diesel::query_builder::AsChangeset;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::member::models::MemberId;
use crate::schema::{member_system_role, system_role};

pub(crate) type SystemRoleId = i32;

/// Stable system roles understood by authorization policy.
#[derive(Clone, Debug, Identifiable, Queryable, Selectable, Serialize)]
#[diesel(table_name = system_role)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct SystemRole {
    pub(crate) id: SystemRoleId,
    pub(crate) name: String,
    pub(crate) authority_level: i32,
    pub(crate) system_managed: bool,
}

/// New system role to be inserted into the database.
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = system_role)]
pub(crate) struct NewSystemRole {
    pub(crate) name: String,
    pub(crate) authority_level: i32,
}

/// Stable platform roles understood by authorization policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BuiltInSystemRole {
    Admin = 50,
    Moderator = 40,
    User = 30,
    Suspended = 10,
    Banned = 0,
}
impl BuiltInSystemRole {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Moderator => "moderator",
            Self::User => "user",
            Self::Suspended => "suspended",
            Self::Banned => "banned",
        }
    }
    pub(crate) fn from_str(name: &str) -> Option<Self> {
        match name {
            "admin" => Some(Self::Admin),
            "moderator" => Some(Self::Moderator),
            "user" => Some(Self::User),
            "suspended" => Some(Self::Suspended),
            "banned" => Some(Self::Banned),
            _ => None,
        }
    }
    pub(crate) const fn authority_level(self) -> i32 {
        match self {
            Self::Admin => 50,
            Self::Moderator => 40,
            Self::User => 30,
            Self::Suspended => 10,
            Self::Banned => 0,
        }
    }
    pub(crate) const fn from_authority_level(level: i32) -> Option<Self> {
        match level {
            50 => Some(Self::Admin),
            40 => Some(Self::Moderator),
            30 => Some(Self::User),
            10 => Some(Self::Suspended),
            0 => Some(Self::Banned),
            _ => None,
        }
    }
}

/// Role set selected during registration.
/// - Default: is User (BuiltInSystemRole::User)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegistrationRoleKind {
    Administrator,
    User,
}
impl RegistrationRoleKind {
    pub(crate) const fn role(self) -> BuiltInSystemRole {
        match self {
            Self::Administrator => BuiltInSystemRole::Admin,
            Self::User => BuiltInSystemRole::User,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RoleIdentifier {
    Id(SystemRoleId),
    Name(String),
}

/// Association between a member and a system role.
#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = member_system_role)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct MemberSystemRole {
    pub(crate) member_id: MemberId,
    pub(crate) role_id: SystemRoleId,
    pub(crate) assigned_by: Option<MemberId>,
    pub(crate) assigned_at: chrono::NaiveDateTime,
}
