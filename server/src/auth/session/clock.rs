use chrono::{DateTime, Duration, NaiveDateTime, Utc};
/// Provides the current UTC time for session expiry and revocation decisions.
///
/// Database-owned audit timestamps such as `created_at` remain the database's
/// responsibility. This clock is only for session business rules.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SessionClock;
impl SessionClock {
    /// Returns the current UTC instant.
    pub(super) fn now() -> DateTime<Utc> {
        Utc::now()
    }

    /// Returns the current UTC instant in the form used by PostgreSQL
    /// `TIMESTAMP` columns mapped to Diesel's `NaiveDateTime`.
    pub(super) fn now_naive() -> NaiveDateTime {
        Self::now().naive_utc()
    }

    /// Returns a UTC expiry instant relative to the current time.
    pub(super) fn expires_in(duration: Duration) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Self::now();
        (now, now + duration)
    }

    /// Returns a PostgreSQL `TIMESTAMP` expiry value relative to the current
    /// time.
    pub(super) fn expires_in_naive(duration: Duration) -> (NaiveDateTime, NaiveDateTime) {
        let now = Self::now_naive();
        (now, now + duration)
    }
}
