use std::{io::Write, sync::Arc};

use crate::argonfs::{
    block_cache::{BlockCache, BlockExclusiveGuard, BlockTag, BlockView, BlockWriter},
    buffer_allocator::{BufferAllocator, BufferHandle},
};

pub struct BlockCacheAllocator {
    block_cache: Arc<BlockCache>,
    block_tag: BlockTag,
    mutable_view: Option<BlockWriter>,
}

impl BlockCacheAllocator {
    pub fn new(block_cache: Arc<BlockCache>, block_tag: BlockTag) -> Self {
        Self {
            block_cache,
            block_tag,
            mutable_view: None,
        }
    }

    pub fn into_block(self) -> BlockExclusiveGuard {
        assert!(self.mutable_view.is_some());
        self.mutable_view.unwrap().into_guard()
    }
}

impl BufferAllocator for BlockCacheAllocator {
    fn alloc(&mut self, buf_size: usize) -> &mut dyn BufferHandle {
        assert!(self.mutable_view.is_none());

        let block = self.block_cache.get_block(&self.block_tag, false);

        let mut block = block.to_exclusive();
        self.block_cache.expand_block(&mut block, buf_size);

        self.mutable_view = Some(BlockWriter::new(block));

        let writer_ref: &mut dyn Write = self.mutable_view.as_mut().unwrap();
        writer_ref
    }
}

unsafe impl Send for BlockCacheAllocator {}

pub struct BlockBufferHandle {
    guard: BlockExclusiveGuard,
}

impl BlockBufferHandle {
    fn new(guard: BlockExclusiveGuard) -> Self {
        Self { guard }
    }
}

impl BufferHandle for BlockBufferHandle {
    fn get_buf(&mut self) -> &mut dyn bytes::Buf {
        BlockView::new(guard)
    }

    fn get_writer(&mut self) -> &mut dyn Write {}
}
