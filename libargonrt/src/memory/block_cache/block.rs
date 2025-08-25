use std::sync::{Mutex, atomic::AtomicU16};

use crate::memory::block_cache::wait_queue::WaitQueue;

struct BlockHeader {
    state: Mutex<BlockHeaderState>,
    ref_count: AtomicU16,
    usage_count: AtomicU16,
}

struct BlockHeaderState {
    state: BlockState,
    next_free: (),
    wait_queue: WaitQueue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockState {
    FREE,
    ACQUIRED,
    LOADED,
}

impl BlockHeader {
    fn acquire(&self) -> Result<(), AcquireError> {
        let state = self.state.lock().unwrap();

        if state.state != BlockState::FREE {
            return Err(AcquireError::BLOCK_NOT_FREE);
        }

        todo!("acquire block");

        Ok(())
    }
}

enum AcquireError {
    BLOCK_NOT_FREE,
}

struct BlockRef {}

impl Drop for BlockRef {
    fn drop(&mut self) {
        todo!()
    }
}
