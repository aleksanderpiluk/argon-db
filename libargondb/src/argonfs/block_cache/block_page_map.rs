use std::ptr::NonNull;

use crate::argonfs::block_cache::page::PageHeader;

#[derive(Debug)]
pub struct BlockPageMap {
    page_map: Box<[(usize, NonNull<u8>, usize)]>,
}

impl BlockPageMap {
    pub unsafe fn new(block: NonNull<PageHeader>) -> (Self, usize) {
        let mut page_map: Vec<(usize, NonNull<u8>, usize)> = vec![];
        let mut pos = 0usize;

        let mut next_block_ptr: Option<NonNull<PageHeader>> = Some(block);

        while let Some(block_ptr) = next_block_ptr {
            let block = unsafe { block_ptr.as_ref() };

            let buf = block.data;
            let buf_len = block.buf_len;

            page_map.push((pos, buf, buf_len));

            pos += buf_len;
            next_block_ptr = block.next_overflow_page();
        }

        (
            Self {
                page_map: page_map.into_boxed_slice(),
            },
            pos,
        )
    }

    pub fn lookup_page_ptr(&self, index: usize) -> (usize, NonNull<u8>, usize) {
        assert!(self.page_map.len() > 0);

        let mut l = 0usize;
        let mut r = self.page_map.len() - 1;

        while l <= r {
            let m = l + ((r - l) / 2);
            let (pos, buf, buf_len) = self.page_map[m];

            if pos < index {
                l = m + 1;
            } else if pos > index {
                r = m - 1;
            } else {
                return (pos, buf, buf_len);
            }
        }

        let (pos, buf, buf_len) = self.page_map[l - 1];

        return (pos, buf, buf_len);
    }
}
