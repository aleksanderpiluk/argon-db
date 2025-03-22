use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};

use dashmap::DashMap;

use super::{Operation, OperationId};

pub struct OperationsStore {
    next_op_id: AtomicU64,
    op_register: DashMap<OperationId, Operation>,
}

impl Default for OperationsStore {
    fn default() -> Self {
        Self {
            next_op_id: AtomicU64::new(1),
            op_register: DashMap::new(),
        }
    }
}

impl OperationsStore {
    pub fn add_op(&self, op: Operation) -> OperationId {
        let op_id = OperationId(self.next_op_id.fetch_add(1, Ordering::SeqCst));

        self.op_register.insert(op_id, op);

        op_id
    }

    pub fn get(&self, op_id: OperationId) -> Option<Operation> {
        self.op_register.remove(&op_id).map(|o| o.1)
    }
}
