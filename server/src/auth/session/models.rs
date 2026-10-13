use chrono::Duration;
use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::member::models::MemberId;
use crate::schema::auth_session;

pub(crate) type SessionId = Uuid;
pub(crate) type SessionFamilyId = Uuid;

/// Represents a new session to be inserted into the database
#[derive(Debug, Insertable)]
#[diesel(table_name = auth_session)]
pub(super) struct NewAuthSession {
    pub id: SessionId,
    pub family_id: SessionFamilyId,
    pub member_id: MemberId,
    pub refresh_token_hash: String,
    pub refresh_expires_at: NaiveDateTime,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

/// Represents a persisted session for a member
#[derive(Debug, Identifiable, Queryable, Selectable)]
#[diesel(table_name = auth_session)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct AuthSession {
    pub id: SessionId,
    pub family_id: SessionFamilyId,
    pub member_id: MemberId,
    pub refresh_token_hash: String,
    pub refresh_expires_at: NaiveDateTime,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: NaiveDateTime,
    pub consumed_at: Option<NaiveDateTime>,
    pub replaced_by: Option<SessionId>,
    pub revoked_at: Option<NaiveDateTime>,
}

/// Represents a configuration for session management, including token lifetimes and security settings
#[derive(Clone, Debug)]
pub(crate) struct SessionConfig {
    pub issuer: String,
    pub audience: String,
    pub access_ttl: Duration,
    pub refresh_ttl: Duration,
    pub cookie_secure: bool,
}

/// Represents metadata associated with a session
#[derive(Clone, Debug, Default)]
pub(crate) struct SessionMetadata {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

/// Trusted identity produced from a valid access token and active session.
#[derive(Clone, Debug)]
pub(crate) struct AuthenticatedPrincipal {
    pub member_id: MemberId,
    pub session_id: SessionId,
}

pub(super) struct RefreshCredentials {
    pub refresh_token: String,
    pub refresh_token_hash: String,
    pub refresh_expires_at: NaiveDateTime,
}
/// Represents an authentication token pair, including the access token, its expiration time, and the refresh token
#[derive(Debug)]
pub(crate) struct AuthCredentials {
    pub access_token: String,  // JWT access token
    pub refresh_token: String, // Refresh token
}
