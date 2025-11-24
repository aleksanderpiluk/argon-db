use crate::argonfs::block_cache::BlockTag;

use super::{
    block_map::BlockMap,
    freelist::Freelist,
    page_buffer::{BlockExclusiveGuard, BlockSharedGuard, PageBuffer},
};

pub struct BlockCache {
    config: BlockCacheConfig,
    buffer: PageBuffer,
    map: BlockMap,
    freelist: Freelist,
}

unsafe impl Send for BlockCache {}
unsafe impl Sync for BlockCache {}

impl BlockCache {
    pub fn new(config: BlockCacheConfig) -> Self {
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

    pub fn get_block(&self, tag: &BlockTag, bump_usage_count: bool) -> BlockSharedGuard {
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

    pub fn expand_block(&self, block: &mut BlockExclusiveGuard, size: usize) {
        assert!(block.is_acquired());

        let page_size = self.config.page_size;
        let required_pages = size.div_ceil(page_size);

        if required_pages == 1 {
            return;
        }

        if let Some(_) = block.next_overflow_page() {
            panic!("block is already expanded");
        }

        let owner_header = block.header();

        let mut overflow_page = self.freelist.get_free_page(&self.buffer, &self.map);
        overflow_page.set_state_overflow_page(owner_header);

        block.set_next_overflow_page(overflow_page.header());

        let mut prev_page = overflow_page;
        for _ in 2..required_pages {
            let mut overflow_page = self.freelist.get_free_page(&self.buffer, &self.map);
            overflow_page.set_state_overflow_page(owner_header);

            prev_page.set_next_overflow_page(overflow_page.header());
            prev_page = overflow_page;
        }
    }
}

pub struct BlockCacheConfig {
    /**
     * Size of page in bytes
     */
    pub page_size: usize,

    /**
     * Number of blocks stored in cache
     */
    pub pages_total: usize,
}
