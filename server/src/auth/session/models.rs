use chrono::Duration;
use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::auth_session;
use crate::member::models::MemberId;

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
    pub expires_at: NaiveDateTime,
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
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
    pub replaced_by: Option<SessionId>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: NaiveDateTime,
    pub last_used_at: NaiveDateTime,
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

/// Represents a session that has been issued, including the new session ID and refresh token
#[derive(Debug)]
pub(crate) struct IssuedSessionData {
    pub session_id: SessionId,
    pub member_id: MemberId,
    pub refresh_token: String,
    pub expires_at: NaiveDateTime,
}

/// Trusted identity produced from a valid access token and active session.
#[derive(Clone, Debug)]
pub(crate) struct AuthenticatedPrincipal {
    pub member_id: MemberId,
    pub session_id: SessionId,
}

/// Represents an authentication token pair, including the access token, its expiration time, and the refresh token
#[derive(Debug)]
pub(crate) struct AuthTokens {
    pub access_token: String,
    pub access_expires_at: i64,
    pub refresh_token: String,
}
