use std::slice;

use bytes::Buf;

use crate::argonfs::block_cache::page::PageState;

use super::block_page_map::BlockPageMap;
use super::page_buffer::BlockSharedGuard;

#[derive(Debug)]
pub struct BlockView {
    guard: BlockSharedGuard,
    page_map: BlockPageMap,
    total_pages_size: usize,
    loaded_block_size: Option<usize>,
    pos: usize,
}

impl BlockView {
    pub fn new(guard: BlockSharedGuard) -> Self {
        let (page_map, total_pages_size) = unsafe { BlockPageMap::new(guard.header()) };
        let loaded_block_size = if let PageState::LoadedBlock { block_size, .. } = guard.state {
            Some(block_size)
        } else {
            None
        };

        Self {
            guard,
            page_map,
            total_pages_size,
            loaded_block_size,
            pos: 0,
        }
    }

    pub fn into_guard(self) -> BlockSharedGuard {
        self.guard
    }
}

impl Buf for BlockView {
    fn remaining(&self) -> usize {
        let total_size = if let Some(block_size) = self.loaded_block_size {
            block_size
        } else {
            self.total_pages_size
        };

        total_size - self.pos
    }

    fn chunk(&self) -> &[u8] {
        let (slice_start, buf_ptr, buf_len) = self.page_map.lookup_page_ptr(self.pos);

        let slice_shift = self.pos - slice_start;
        let slice_len = buf_len - slice_shift;

        let slice = unsafe { slice::from_raw_parts(buf_ptr.as_ptr().add(slice_shift), slice_len) };
        slice
    }

    fn advance(&mut self, cnt: usize) {
        if cnt > self.remaining() {
            panic!("cnt is greater than remaining");
        }

        self.pos += cnt;
    }
}

unsafe impl Send for BlockView {}
unsafe impl Sync for BlockView {}
