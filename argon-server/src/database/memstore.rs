use std::sync::atomic::{AtomicUsize, Ordering};

use bytes::Bytes;
use crossbeam_skiplist::SkipSet;

use super::data_types::Cell;

// pub struct Memstore {
//     active_segment: Box<Segment>,
//     in_flush_segments: SkipSet<Box<Segment>>,
// }

#[derive(Debug)]
enum SegmentError {
    SegmentSizeExceeded,
}

#[derive(Debug)]
pub struct Segment {
    id: Bytes,
    store: SkipSet<Cell>,
    size: AtomicUsize,
    max_size: usize,
    // mvcc_read_id: AtomicUsize,
    // flush_lock: AtomicBool,
    // op_barrier: AtomicU8,
}

impl Segment {
    pub fn try_insert(&self, cell: Cell) -> Result<(), SegmentError> {
        let cell_len = cell.len();

        loop {
            let current_size = self.size.load(Ordering::Relaxed);
            let new_size = current_size + cell_len;
            if new_size > self.max_size {
                return Err(SegmentError::SegmentSizeExceeded);
            }

            if let Ok(_) = self.size.compare_exchange(
                current_size,
                new_size,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                return Ok(());
            }
        }
    }
}
