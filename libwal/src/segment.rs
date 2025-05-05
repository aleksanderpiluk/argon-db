use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::platform::{drop_segment, sync_segment};

pub struct Segment<'a> {
    pub(crate) data: &'a mut [u8],
    pub(crate) segment_size: u64,
    pub(crate) synced_ptr: u64,
    pub(crate) tail_ptr: u64,
}

impl<'a> Segment<'a> {
    pub const SEGMENT_LENGTH: u64 = 16; // * 1024 * 1024;

    pub(crate) fn write(&mut self, data: &[u8]) -> Result<(), ()> {
        let tail_ptr = self.tail_ptr;
        let data_len = data.len() as u64;

        let next_tail = tail_ptr + data_len;
        if next_tail >= self.segment_size {
            todo!("Result error")
        }

        self.data[tail_ptr as usize..].copy_from_slice(data);

        self.tail_ptr = next_tail;
        Ok(())
    }

    pub(crate) fn sync(&mut self) {
        sync_segment(self);
    }
}

impl<'a> Drop for Segment<'a> {
    fn drop(&mut self) {
        drop_segment(self);
    }
}

pub(crate) struct SegmentId;
