use std::{io::Write, slice};

use crate::argonfs::block_cache::BlockExclusiveGuard;

use super::block_page_map::BlockPageMap;

pub struct BlockWriter {
    guard: BlockExclusiveGuard,
    page_map: BlockPageMap,
    pos: usize,
}

impl BlockWriter {
    pub fn new(guard: BlockExclusiveGuard) -> Self {
        let (page_map, block_len) = unsafe { BlockPageMap::new(guard.header()) };
        Self {
            guard,
            page_map,
            pos: 0,
        }
    }

    pub fn into_guard(self) -> BlockExclusiveGuard {
        self.guard
    }
}

impl Write for BlockWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buf_pos = 0usize;

        while buf_pos < buf.len() {
            let buf_remaining = buf.len() - buf_pos;

            let (slice_start, buf_ptr, buf_len) = self.page_map.lookup_page_ptr(self.pos);
            let slice = unsafe { slice::from_raw_parts_mut(buf_ptr.as_ptr(), buf_len) };

            let slice_pos = self.pos - slice_start;
            let slice_remaining = slice.len() - slice_pos;
            if slice_remaining == 0 {
                break;
            }

            let write_len = usize::min(slice_remaining, buf_remaining);

            slice[slice_pos..(slice_pos + write_len)]
                .copy_from_slice(&buf[buf_pos..(buf_pos + write_len)]);

            buf_pos += write_len;
            self.pos += write_len;
        }

        Ok(buf_pos)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        panic!("flush not supported")
    }
}
