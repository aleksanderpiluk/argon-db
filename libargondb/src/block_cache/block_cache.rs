use crate::block_cache::{
    block_buffer::{BlockBuffer, BlockExclusiveGuard, BlockSharedGuard},
    block_map::BlockMap,
    freelist::Freelist,
};

pub struct BlockCache {
    config: BlockCacheConfig,
    buffer: BlockBuffer,
    map: BlockMap,
    freelist: Freelist,
}

impl BlockCache {
    fn new(config: BlockCacheConfig) -> Self {
        let buffer = BlockBuffer::new(&config);
        let map = BlockMap::new(&config);
        let freelist = Freelist::new(&buffer);
        Self {
            config,
            buffer,
            map,
            freelist,
        }
    }

    pub fn get_block(&self, tag: u64, bump_usage_count: bool) -> BlockSharedGuard {
        loop {
            if let Some(block) = self.map.get_shared(tag, bump_usage_count) {
                return block;
            }

            let block: BlockExclusiveGuard = self.freelist.get_free_page(&self.buffer, &self.map);
            if let Err(block) = self.map.try_assign_tag(tag, block) {
                self.freelist.push_free(block);
            }
        }
    }
}

pub struct BlockCacheConfig {
    /**
     * Size of block in bytes
     */
    pub block_size: usize,

    /**
     * Number of blocks stored in cache
     */
    pub blocks_total: usize,
}
