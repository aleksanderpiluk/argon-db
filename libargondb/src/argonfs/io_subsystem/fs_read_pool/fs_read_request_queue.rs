use crossbeam::queue::ArrayQueue;

use super::FsReadRequest;

pub struct FsReadRequestQueue {
    inner: ArrayQueue<FsReadRequest>,
}

impl FsReadRequestQueue {
    pub fn new() -> Self {
        Self {
            inner: ArrayQueue::new(64),
        }
    }

    pub fn push(&self, read_request: FsReadRequest) {
        self.inner.push(read_request);
    }
}
