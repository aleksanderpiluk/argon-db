use std::sync::Arc;

use super::wal_state::*;

pub struct WALManager {
    wal_state: Arc<WALState>,
}

impl WALManager {
    pub fn new(wal_state: Arc<WALState>) -> Self {
        Self { wal_state }
    }
}
