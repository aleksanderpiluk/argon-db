use std::{
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::Waker,
};

use heapless::Vec;

use crate::limits::WAIT_QUEUE_CAPACITY;

pub struct PageHeader {
    ref_count: AtomicUsize,
    page_id: PageId,
    page_tag: PageTag,
    inner: Mutex<PageHeaderInnerState>,
}

pub struct PageHeaderInnerState {
    wait_queue: Vec<Waker, WAIT_QUEUE_CAPACITY>,
}

impl PageHeader {
    #[inline]
    pub fn ref_count_increment(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn ref_count_decrement(&self) {
        self.ref_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn call_wakers(&self) {
        let inner = self.inner.lock().unwrap();
        for waker in inner.wait_queue {
            waker.wake();
        }
    }
}

struct PageId(u64);

pub struct PageTag;
