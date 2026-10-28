use std::sync::{Arc, Mutex};

use uuid::{ContextV7, Timestamp, Uuid};

/// Thread-safe application UUID generator.
///
/// `ContextV7` maintains a monotonic counter using interior mutability, but
/// it is intentionally not `Sync`. The mutex serializes access to that
/// counter while the outer `Arc` makes the generator cheap to clone into
/// application services.
#[derive(Clone)]
pub struct IdGenerator {
    context: Arc<Mutex<ContextV7>>,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            context: Arc::new(Mutex::new(ContextV7::new())),
        }
    }

    pub fn generate_uuid(&self) -> Uuid {
        let context = self
            .context
            .lock()
            .expect("UUID generator context mutex was poisoned");

        Uuid::new_v7(Timestamp::from_unix(&*context, 0, 0))
    }
}
