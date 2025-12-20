use std::sync::atomic::{AtomicU64, Ordering};

use super::ObjectId;

#[derive(Debug)]
pub struct ObjectIdGenerator {
    next_id: AtomicU64,
}

impl ObjectIdGenerator {
    pub fn new(state: u64) -> Self {
        Self {
            next_id: AtomicU64::new(state),
        }
    }

    pub fn next(&self) -> ObjectId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        ObjectId(id)
    }

    pub fn state(&self) -> u64 {
        self.next_id.load(Ordering::SeqCst)
    }
}
