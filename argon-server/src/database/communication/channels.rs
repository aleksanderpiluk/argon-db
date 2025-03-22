use crossbeam::queue::SegQueue;

use crate::database::operations::OperationId;

pub struct Channels {
    operations: SegQueue<OperationId>,
    operation_results: SegQueue<()>,
    flush_requests: SegQueue<()>,
}

impl Default for Channels {
    fn default() -> Self {
        Self {
            operations: SegQueue::new(),
            operation_results: SegQueue::new(),
            flush_requests: SegQueue::new(),
        }
    }
}

impl Channels {
    pub fn operations(&self) -> &SegQueue<OperationId> {
        &self.operations
    }
}
