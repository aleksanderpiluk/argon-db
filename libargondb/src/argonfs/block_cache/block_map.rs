use std::{collections::HashMap, ptr::NonNull, sync::Mutex};

use crate::argonfs::block_cache::BlockTag;

use super::{
    block_cache::BlockCacheConfig,
    page::PageHeader,
    page_buffer::{BlockExclusiveGuard, BlockSharedGuard},
};

pub struct BlockMap {
    inner: Mutex<HashMap<BlockTag, NonNull<PageHeader>>>,
}

impl BlockMap {
    pub fn new(config: &BlockCacheConfig) -> Self {
        Self {
            inner: Mutex::new(HashMap::with_capacity(config.pages_total)),
        }
    }

    pub fn try_assign_tag(
        &self,
        tag: &BlockTag,
        mut block: BlockExclusiveGuard,
    ) -> Result<(), BlockExclusiveGuard> {
        assert!(block.is_free());

        let mut map = self.inner.lock().unwrap();

        if let Some(_) = map.get(tag) {
            // Other page is already assigned
            return Err(block);
        }

        map.insert(*tag, block.header());
        block.set_state_acquired(*tag);

        drop(block);
        Ok(())
    }

    /**
     * Tries to free loaded block. If succeed returns BlockExclusiveGuard to block in "Free" state. In case of error - function caller must drop BlockExclusiveGuard to prevent possible deadlock.
     */
    pub fn try_free_loaded_block(
        &self,
        mut block: BlockExclusiveGuard,
    ) -> Result<(BlockExclusiveGuard, Option<NonNull<PageHeader>>), BlockExclusiveGuard> {
        assert!(block.is_loaded_block());

        // If other thread tries to obtain lock on the same page with map locked, then blocking would result in deadlock.
        let mut map = match self.inner.try_lock() {
            Ok(map) => map,
            Err(e) => match e {
                std::sync::TryLockError::WouldBlock => return Err(block),
                std::sync::TryLockError::Poisoned(e) => panic!("map mutex poisoned: {}", e), // FIXME: This should be handled in a better way
            },
        };

        let (tag, next_overflow_page) = block.set_state_free_from_loaded_block();
        assert!(block.is_free());

        // We can remove page from map
        let r = map.remove(&tag);
        assert!(r.is_some());

        Ok((block, next_overflow_page))
    }

    pub fn get_exclusive(&self, tag: BlockTag) -> Option<BlockExclusiveGuard> {
        let map = self.inner.lock().unwrap();

        let header = match map.get(&tag) {
            Some(header) => *header,
            None => return None,
        };

        // Because map lock is obtained, we can assume that tag can't change during acquisition
        let block = unsafe { BlockExclusiveGuard::acquire_for(header) };

        assert!(block.is_loaded_block() || block.is_acquired());
        Some(block)
    }

    pub fn get_shared(&self, tag: &BlockTag, bump_usage_count: bool) -> Option<BlockSharedGuard> {
        let map = self.inner.lock().unwrap();

        let header = match map.get(&tag) {
            Some(header) => *header,
            None => return None,
        };

        // Because map lock is obtained, we can assume that tag can't change during acquisition
        let block = unsafe { BlockSharedGuard::acquire_for(header, bump_usage_count) };

        assert!(block.is_loaded_block() || block.is_acquired());
        Some(block)
    }
}
