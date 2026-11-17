//! Shared synchronization for tests that inspect or mutate process environment.

use std::env;
use std::ffi::OsString;
use std::sync::{Mutex, MutexGuard, OnceLock};

/// Environment variables consumed while constructing application configuration.
pub(crate) const CONFIG_ENVIRONMENT: &[&str] = &[
    "APP_ENV",
    "DATABASE_URL",
    "JWT_SECRET",
    "ADMINISTRATION_KEY",
    "COOKIE_SECURE",
    "ALLOWED_ORIGINS",
];

fn environment_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn system_role_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Serializes access to process environment and restores captured values on drop.
pub(crate) struct EnvironmentCheckpoint {
    original: Vec<(String, Option<OsString>)>,
    _guard: MutexGuard<'static, ()>,
}

impl EnvironmentCheckpoint {
    /// Captures the listed variables while holding the global environment lock.
    pub(crate) fn capture(names: &[&str]) -> Self {
        let guard = environment_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let original = names
            .iter()
            .map(|name| ((*name).to_string(), env::var_os(name)))
            .collect();
        Self {
            original,
            _guard: guard,
        }
    }

    /// Sets one variable for the lifetime of this checkpoint.
    pub(crate) fn set(&self, name: &str, value: &str) {
        env::set_var(name, value);
    }

    /// Removes one variable for the lifetime of this checkpoint.
    pub(crate) fn remove(&self, name: &str) {
        env::remove_var(name);
    }
}

impl Drop for EnvironmentCheckpoint {
    fn drop(&mut self) {
        for (name, value) in &self.original {
            match value {
                Some(value) => env::set_var(name, value),
                None => env::remove_var(name),
            }
        }
    }
}

/// Serializes tests that mutate shared system-role assignment state.
pub(crate) struct SystemRoleCheckpoint {
    _guard: MutexGuard<'static, ()>,
}

impl SystemRoleCheckpoint {
    /// Acquires the role-state checkpoint, recovering from prior test panics.
    pub(crate) fn acquire() -> Self {
        Self {
            _guard: system_role_lock()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }
}
