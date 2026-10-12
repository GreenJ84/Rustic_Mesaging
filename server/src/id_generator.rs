use std::sync::Arc;
use uuid::{ContextV7, Timestamp, Uuid};

#[derive(Clone)]
pub struct IdGenerator {
    context: Arc<ContextV7>,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            context: Arc::new(ContextV7::new()),
        }
    }

    pub fn generate_uuid(&self) -> Uuid {
        Uuid::new_v7(Timestamp::from_unix(&*self.context, 0u64, 0u32))
    }
}
