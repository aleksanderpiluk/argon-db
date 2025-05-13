use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam_skiplist::SkipSet;

struct Segment {
    schema: SegmentSchema,
    data: SkipSet<SegmentCell>,
    size: AtomicUsize,
    size_limit: usize,
}

impl Segment {
    fn try_insert(&self, cell: SegmentCell) -> Result<(), ()> {
        let cell_size = cell.size();

        loop {
            let segment_size = self.size.load(Ordering::Acquire);
            let next_segment_size = segment_size + cell_size;

            if next_segment_size > self.size_limit {
                return Err(());
            }

            if let Ok(_) = self.size.compare_exchange(
                segment_size,
                next_segment_size,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                return Ok(());
            }
        }
    }
}

struct SegmentSchema {}

struct SegmentCell {}

impl SegmentCell {
    fn size(&self) -> usize {
        todo!()
    }
}
