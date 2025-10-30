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
    }
}
