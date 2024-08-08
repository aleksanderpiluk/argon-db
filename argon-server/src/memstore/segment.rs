use std::sync::atomic::{AtomicBool, AtomicUsize};

use crossbeam_skiplist::SkipSet;

pub struct Segment {
    id: String,
    flush_lock: AtomicBool,
    read_count: AtomicUsize,
    data_size: AtomicUsize,
    store: SkipSet<()>,
}

impl Segment {
    pub fn new(id: String) -> Self {
        Self {
            id,
            flush_lock: AtomicBool::new(false),
            read_count: AtomicUsize::new(0),
            data_size: AtomicUsize::new(0),
            store: SkipSet::new(),
        }
    }
}
