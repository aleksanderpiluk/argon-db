use std::{
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
};

/**
 * Memtable lock provides access control for writers, readers and flushers.
 * When memtable is moved into read-only state and all writes are finished, memtable is flush-ready.
 */
pub struct MemtableLock(AtomicUsize);

impl MemtableLock {
    const INITIAL_STATE: usize = 0;

    const FLAG_READ_ONLY: usize = 1 << 63;

    pub fn new() -> Self {
        Self(AtomicUsize::new(Self::INITIAL_STATE))
    }

    pub fn obtain_write_access(&self) -> Result<(), ()> {
        loop {
            let state = self.0.load(Ordering::Acquire);

            let is_read_only = (state & Self::FLAG_READ_ONLY) > 0;
            if is_read_only {
                return Err(());
            }

            // TODO: Add debug assertions
            let new_state = state + 1;

            if let Ok(_) =
                self.0
                    .compare_exchange(state, new_state, Ordering::Release, Ordering::Relaxed)
            {
                return Ok(());
            }
        }
    }

    pub fn release_write_access(&self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }

    pub fn enable_read_only_mode(&self) {
        self.0.fetch_xor(Self::FLAG_READ_ONLY, Ordering::AcqRel);
    }

    pub fn is_flush_ready(&self) -> bool {
        let state = self.0.load(Ordering::Acquire);

        return state == Self::FLAG_READ_ONLY;
    }
}

impl Debug for MemtableLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.0.load(Ordering::Acquire);

        let is_read_only = (state & Self::FLAG_READ_ONLY) > 0;
        let writers_count = state & (!Self::FLAG_READ_ONLY);

        f.debug_struct("MemtableLock")
            .field("read_only", &is_read_only)
            .field("writers_count", &writers_count)
            .finish()
    }
}
