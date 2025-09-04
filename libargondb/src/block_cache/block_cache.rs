use crate::block_cache::{
    block_buffer::{BlockBuffer, BlockExclusiveGuard, BlockSharedGuard},
    block_map::BlockMap,
    freelist::Freelist,
};

pub struct BlockCache {
    buffer: BlockBuffer,
    map: BlockMap,
    freelist: Freelist,
}

impl BlockCache {
    fn new() -> Self {
        todo!()
    }

    fn get_block(&self, tag: u64) -> BlockSharedGuard {
        loop {
            if let Some(block) = self.map.get_shared(tag) {
                return block;
            }

            let block: BlockExclusiveGuard = self.freelist.get_free_page(&self.buffer, &self.map);
            if let Err(block) = self.map.try_assign_tag(tag, block) {
                self.freelist.push_free(block);
            }
        }
    }
}
