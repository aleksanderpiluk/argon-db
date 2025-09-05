use std::{collections::HashMap, ptr::NonNull, sync::Mutex};

use crate::block_cache::{
    block_buffer::{BlockExclusiveGuard, BlockHeader, BlockSharedGuard},
    block_cache::BlockCacheConfig,
};

pub struct BlockMap {
    inner: Mutex<HashMap<u64, NonNull<BlockHeader>>>,
}

impl BlockMap {
    pub fn new(config: &BlockCacheConfig) -> Self {
        Self {
            inner: Mutex::new(HashMap::with_capacity(config.blocks_total)),
        }
    }

    pub fn try_assign_tag(
        &self,
        tag: u64,
        mut block: BlockExclusiveGuard,
    ) -> Result<(), BlockExclusiveGuard> {
        assert!(block.is_free());
        assert_eq!(block.next_free(), None);

        let mut map = self.inner.lock().unwrap();

        if let Some(_) = map.get(&tag) {
            // Other page is already assigned
            return Err(block);
        }

        map.insert(tag, block.header());
        block.acquire(tag);

        drop(block);
        Ok(())
    }

    /**
     * Tries to remove tag assigned to block and put it in the free state. If succeed returns BlockExclusiveGuard to freed block. In case of error, other thread tries to access
     * this block, which means it's still in use - function caller must drop BlockExclusiveGuard to prevent deadlock.
     */
    pub fn try_free_assigned_block(
        &self,
        mut block: BlockExclusiveGuard,
    ) -> Result<BlockExclusiveGuard, BlockExclusiveGuard> {
        let mut map = match self.inner.try_lock() {
            Ok(map) => map,
            Err(e) => match e {
                std::sync::TryLockError::WouldBlock => return Err(block),
                std::sync::TryLockError::Poisoned(e) => panic!("map mutex poisoned: {}", e), // FIXME: This should be handled in a better way
            },
        };

        assert!(block.is_loaded());
        assert_eq!(block.next_free(), None);

        let tag = block.tag().unwrap();

        let r = map.remove(&tag);
        assert!(r.is_some());

        // At this point block is removed from map so we can free it
        block.free();
        assert!(block.is_free());
        assert_eq!(block.next_free(), None);

        Ok(block)
    }

    pub fn get_exclusive(&self, tag: u64) -> Option<BlockExclusiveGuard> {
        let map = self.inner.lock().unwrap();

        let header = match map.get(&tag) {
            Some(header) => *header,
            None => return None,
        };

        // Because map lock is obtained, we can assume that tag can't change during acquisition
        let block = unsafe { BlockExclusiveGuard::acquire_for(header) };

        assert!(!block.is_free());
        assert_eq!(block.next_free(), None);
        assert_eq!(block.tag(), Some(tag));

        Some(block)
    }

    pub fn get_shared(&self, tag: u64, bump_usage_count: bool) -> Option<BlockSharedGuard> {
        let map = self.inner.lock().unwrap();

        let header = match map.get(&tag) {
            Some(header) => *header,
            None => return None,
        };

        // Because map lock is obtained, we can assume that tag can't change during acquisition
        let block = unsafe { BlockSharedGuard::acquire_for(header, bump_usage_count) };

        assert!(!block.is_free());
        assert_eq!(block.next_free(), None);
        assert_eq!(block.tag(), Some(tag));

        Some(block)
    }
}
