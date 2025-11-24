use std::{collections::HashMap, ptr::NonNull, sync::Mutex};

<<<<<<< HEAD:libargondb/src/block_cache/block_map.rs
use crate::block_cache::{
    page_buffer::{BlockExclusiveGuard, BlockSharedGuard, PageHeader},
    block_cache::{BlockCacheConfig, BlockCacheTag},
};

pub struct BlockMap {
    inner: Mutex<HashMap<BlockCacheTag, NonNull<PageHeader>>>,
=======
use crate::argonfs::block_cache::BlockTag;

use super::{
    block_cache::BlockCacheConfig,
    page::PageHeader,
    page_buffer::{BlockExclusiveGuard, BlockSharedGuard},
};

pub struct BlockMap {
    inner: Mutex<HashMap<BlockTag, NonNull<PageHeader>>>,
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/block_map.rs
}

impl BlockMap {
    pub fn new(config: &BlockCacheConfig) -> Self {
        Self {
            inner: Mutex::new(HashMap::with_capacity(config.pages_total)),
        }
    }

    pub fn try_assign_tag(
        &self,
<<<<<<< HEAD:libargondb/src/block_cache/block_map.rs
        tag: &BlockCacheTag,
=======
        tag: &BlockTag,
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/block_map.rs
        mut block: BlockExclusiveGuard,
    ) -> Result<(), BlockExclusiveGuard> {
        assert!(block.is_free());

        let mut map = self.inner.lock().unwrap();

        if let Some(_) = map.get(tag) {
            // Other page is already assigned
            return Err(block);
        }

        map.insert(*tag, block.header());
<<<<<<< HEAD:libargondb/src/block_cache/block_map.rs
        block.acquire(*tag);
=======
        block.set_state_acquired(*tag);
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/block_map.rs

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

<<<<<<< HEAD:libargondb/src/block_cache/block_map.rs
    pub fn get_exclusive(&self, tag: BlockCacheTag) -> Option<BlockExclusiveGuard> {
=======
    pub fn get_exclusive(&self, tag: BlockTag) -> Option<BlockExclusiveGuard> {
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/block_map.rs
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

<<<<<<< HEAD:libargondb/src/block_cache/block_map.rs
    pub fn get_shared(
        &self,
        tag: &BlockCacheTag,
        bump_usage_count: bool,
    ) -> Option<BlockSharedGuard> {
=======
    pub fn get_shared(&self, tag: &BlockTag, bump_usage_count: bool) -> Option<BlockSharedGuard> {
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/block_map.rs
        let map = self.inner.lock().unwrap();

        let header = match map.get(&tag) {
            Some(header) => *header,
            None => return None,
        };

        // Because map lock is obtained, we can assume that tag can't change during acquisition
        let block = unsafe { BlockSharedGuard::acquire_for(header, bump_usage_count) };

<<<<<<< HEAD:libargondb/src/block_cache/block_map.rs
        assert!(!block.is_free());
        assert_eq!(block.next_free(), None);
        assert_eq!(block.tag(), Some(*tag));

=======
        assert!(block.is_loaded_block() || block.is_acquired());
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/block_map.rs
        Some(block)
    }
}
