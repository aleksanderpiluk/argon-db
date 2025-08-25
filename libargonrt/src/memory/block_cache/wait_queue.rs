use std::task::Waker;

use heapless::Vec;

use crate::limits::WAIT_QUEUE_CAPACITY;

pub(crate) struct WaitQueue {
    wait_queue: Vec<Waker, { WAIT_QUEUE_CAPACITY }>,
}
