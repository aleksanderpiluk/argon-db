use std::sync::atomic::AtomicU8;

use crossbeam::channel::{Receiver, Sender, bounded};

use super::BlockPointer;

#[derive(Debug, Clone)]
pub struct ReadQueue {
    inner: (Sender<ReadRequest>, Receiver<ReadRequest>),
}

impl ReadQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: bounded(capacity),
        }
    }

    pub fn pop(&self) -> Option<ReadRequest> {
        let (sender, receiver) = &self.inner;
        match receiver.recv() {
            Ok(request) => Some(request),
            Err(err) => panic!("ReadQueue pop panicked with error {}", err), // TODO: Revise error handling
        }
    }

    pub fn push(&self, msg: ReadRequest) {
        let (sender, receiver) = &self.inner;
        match sender.send(msg) {
            Ok(_) => {}
            Err(err) => panic!("ReadQueue push panicked with error {}", err), // TODO: Revise error handling
        }
    }
}

pub(crate) struct ReadRequest {
    block_ptr: BlockPointer,
    page: *mut [u8],
    state: AtomicU8,
}

impl ReadRequest {
    fn new() -> Self {}
}
