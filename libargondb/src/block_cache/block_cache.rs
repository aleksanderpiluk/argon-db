use crate::{
    block_cache::{
        block_map::BlockMap,
        freelist::Freelist,
        page_buffer::{BlockExclusiveGuard, BlockSharedGuard, PageBuffer},
    },
    kv::KVSSTableBlockPtr,
};

pub type BlockCacheTag = KVSSTableBlockPtr;

pub struct BlockCache {
    config: BlockCacheConfig,
    buffer: PageBuffer,
    map: BlockMap,
    freelist: Freelist,
}

impl BlockCache {
    fn new(config: BlockCacheConfig) -> Self {
        let buffer = PageBuffer::new(&config);
        let map = BlockMap::new(&config);
        let freelist = Freelist::new(&buffer);
        Self {
            config,
            buffer,
            map,
            freelist,
        }
    }

    pub fn get_block(&self, tag: &BlockCacheTag, bump_usage_count: bool) -> BlockSharedGuard {
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
