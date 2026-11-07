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

        Uuid::new_v7(Timestamp::now(&*context))
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::IdGenerator;

    #[test]
    fn generates_distinct_uuid_v7_values() {
        let generator = IdGenerator::new();
        let first = generator.generate_uuid();
        let second = generator.generate_uuid();

        assert_ne!(first, second);
        assert_eq!(first.get_version_num(), 7);
        assert_eq!(second.get_version_num(), 7);
    }

    #[test]
    fn remains_safe_when_shared_across_threads() {
        let generator = IdGenerator::new();
        let handles = (0..8)
            .map(|_| {
                let generator = generator.clone();
                thread::spawn(move || generator.generate_uuid())
            })
            .collect::<Vec<_>>();

        let ids = handles
            .into_iter()
            .map(|handle| handle.join().expect("ID generation thread should finish"))
            .collect::<std::collections::HashSet<_>>();

        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn generates_a_large_unique_batch() {
        let generator = IdGenerator::new();
        let ids = (0..1_000)
            .map(|_| generator.generate_uuid())
            .collect::<std::collections::HashSet<_>>();

        assert_eq!(ids.len(), 1_000);
    }

    #[test]
    fn clones_share_generation_context() {
        let generator = IdGenerator::new();
        let clone = generator.clone();

        assert_ne!(generator.generate_uuid(), clone.generate_uuid());
    }

    #[test]
    fn sequential_ids_are_time_ordered() {
        let generator = IdGenerator::new();
        let first = generator.generate_uuid();
        let second = generator.generate_uuid();

        assert!(first < second);
    }
}
