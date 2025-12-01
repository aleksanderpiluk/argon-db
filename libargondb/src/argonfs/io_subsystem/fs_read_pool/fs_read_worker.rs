use std::sync::Arc;

use crate::argonfs::{
    block_cache::{BlockCache, BlockCacheAllocator},
    io_subsystem::{FsReadRequest, fs_read_pool::FsReadRequestQueue},
};

struct FsReadWorker {
    block_cache: Arc<BlockCache>,
    fs_read_request_queue: Arc<FsReadRequestQueue>,
}

impl FsReadWorker {
    fn run(self) {}

    fn execute_read_request(&self, read_request: FsReadRequest) {
        let block_cache = self.block_cache.clone();
        let format_reader = read_request.sstable_format_reader;
        let block_tag = read_request.block_tag;
        let sstable_ptr = read_request.sstable_ptr;

        let mut block_alloc = BlockCacheAllocator::new(block_cache.clone(), block_tag);
        let block_size = format_reader.load_data_block(sstable_ptr, &mut block_alloc);

        // let mut block = block_alloc.into_block();
        let mut block = block_cache.get_block(&block_tag, false).to_exclusive();
        let wakers = block.set_state_loaded_block(block_size);
        for waker in wakers {
            waker.wake();
        }
    }
}
