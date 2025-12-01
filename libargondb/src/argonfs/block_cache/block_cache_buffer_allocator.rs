use std::sync::Arc;

use crate::argonfs::{
    block_cache::{BlockCache, BlockTag},
    core::{BufferAllocator, BufferHandle},
};

pub struct BlockCacheAllocator {
    block_cache: Arc<BlockCache>,
    block_tag: BlockTag,
}

impl BlockCacheAllocator {
    pub fn new(block_cache: Arc<BlockCache>, block_tag: BlockTag) -> Self {
        Self {
            block_cache,
            block_tag,
        }
    }
}

impl BufferAllocator for BlockCacheAllocator {
    fn alloc(&mut self, buf_size: usize) -> Box<dyn BufferHandle> {
        let block = self.block_cache.get_block(&self.block_tag, false);

        let mut block = block.to_exclusive();
        self.block_cache.expand_block(&mut block, buf_size);

        Box::new(block)
    }
}

// pub struct BlockBufferHandle {
//     guard: BlockExclusiveGuard,
// }

// impl BlockBufferHandle {
//     fn new(guard: BlockExclusiveGuard) -> Self {
//         Self { guard }
//     }
// }

// impl BufferHandle for BlockBufferHandle {
//     fn get_buf(&mut self) -> &mut dyn bytes::Buf {
//         todo!()
//     }

//     fn get_writer(&mut self) -> Box<dyn Write> {
//         BlockWriter::new(self.guard)
//     }
// }
