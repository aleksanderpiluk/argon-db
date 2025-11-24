<<<<<<< HEAD:libargondb/src/block_cache/block_view.rs
use std::{ops::Index, ptr::NonNull};

use crate::block_cache::page_buffer::BlockSharedGuard;

pub struct BlockView<'a> {
    guard: BlockSharedGuard,
    page_map: Vec<(usize, &'a [u8])>,
}

impl<'a> BlockView<'a> {
    pub fn from(guard: BlockSharedGuard) -> Self {
        let page_map = todo!();
        Self { guard, page_map }
    }

    fn lookup_slice(&self, index: usize) -> (usize, &'a [u8]) {
        assert!(self.page_map.len() > 0);

        todo!("binary search");
    }
}

impl<'a> Index<usize> for BlockView<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        let (index_base, slice) = self.lookup_slice(index);

        &slice[index - index_base]
=======
use std::slice;

use bytes::Buf;

use super::block_page_map::BlockPageMap;
use super::page_buffer::BlockSharedGuard;

pub struct BlockView {
    guard: BlockSharedGuard,
    page_map: BlockPageMap,
    block_len: usize,
    pos: usize,
}

impl BlockView {
    pub fn new(guard: BlockSharedGuard) -> Self {
        let (page_map, block_len) = unsafe { BlockPageMap::new(guard.header()) };
        Self {
            guard,
            page_map,
            block_len,
            pos: 0,
        }
    }

    pub fn into_guard(self) -> BlockSharedGuard {
        self.guard
    }
}

impl Buf for BlockView {
    fn remaining(&self) -> usize {
        self.block_len - self.pos
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
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/block_view.rs
    }
}
