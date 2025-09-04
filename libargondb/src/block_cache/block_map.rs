use std::{collections::HashMap, ptr::NonNull, sync::Mutex};

use crate::block_cache::block_buffer::{BlockExclusiveGuard, BlockHeader, BlockSharedGuard};

pub struct BlockMap {
    inner: Mutex<HashMap<u64, NonNull<BlockHeader>>>,
}

impl BlockMap {
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

    pub fn free_assigned_block(&self, mut block: BlockExclusiveGuard) -> BlockExclusiveGuard {
        let mut map = self.inner.lock().unwrap();

        assert!(block.is_loaded());
        assert_eq!(block.next_free(), None);

        let tag = block.tag().unwrap();

        let r = map.remove(&tag);
        assert!(r.is_some());

        // At this point block is removed from map so we can free it
        block.free();
        assert!(block.is_free());
        assert_eq!(block.next_free(), None);

        block
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

    pub fn get_shared(&self, tag: u64) -> Option<BlockSharedGuard> {
        let map = self.inner.lock().unwrap();

        let header = match map.get(&tag) {
            Some(header) => *header,
            None => return None,
        };

        // Because map lock is obtained, we can assume that tag can't change during acquisition
        let block = unsafe { BlockSharedGuard::acquire_for(header) };

        assert!(!block.is_free());
        assert_eq!(block.next_free(), None);
        assert_eq!(block.tag(), Some(tag));

        Some(block)
    }
}
