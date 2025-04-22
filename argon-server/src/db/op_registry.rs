// use std::sync::{
//     atomic::{AtomicU64, Ordering},
//     OnceLock,
// };

// use dashmap::DashMap;

// pub struct OpRegistry {
//     next_op_id: AtomicU64,
//     op_register: DashMap<u64, OpRegistryItem>,
// }

// pub fn op_registry() -> &'static OpRegistry {
//     static OP_REGISTRY: OnceLock<OpRegistry> = OnceLock::new();
//     OP_REGISTRY.get_or_init(|| OpRegistry {
//         next_op_id: AtomicU64::new(1),
//         op_register: DashMap::new(),
//     })
// }

// impl OpRegistry {
//     pub fn add_op(&self, callback: Box<dyn FnOnce() + Send + Sync>) -> u64 {
//         let op_id = self.next_op_id.fetch_add(1, Ordering::SeqCst);

//         self.op_register.insert(op_id, OpRegistryItem { callback });

//         op_id
//     }

//     pub fn get(&self, op_id: u64) -> Option<(u64, OpRegistryItem)> {
//         self.op_register.remove(&op_id)
//     }
// }
// pub struct OpRegistryItem {
//     pub callback: Box<dyn FnOnce() + Send + Sync>,
// }
