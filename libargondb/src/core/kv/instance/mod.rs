mod state;

use flume::{Receiver, Sender};
use std::sync::Arc;

use crate::{
    kv::{
        KVRuntimeError, KVRuntimeErrorKind, ObjectId, Table,
        config::KVConfig,
        memtable::{KVMemtableFlushRequest, Memtable},
        object_id::ObjectIdGenerator,
    },
    utils::rcu::RCU,
};

pub struct Instance {}

impl Instance {}

#[derive(Debug)]
pub struct KVInstance {
    config: KVConfig,
    object_id_generator: ObjectIdGenerator,
    state: RCU<KVInstanceState>,
}

impl KVInstance {
    pub fn new(config: KVConfig, initial_state: KVInstanceStateSnapshot) -> Self {
        Self {
            config,
            object_id_generator: ObjectIdGenerator::new(initial_state.object_id_generator_state),
            state: RCU::new(Arc::new(KVInstanceState::Active {
                memtable_flush_queue: MemtableFlushQueue::new(),
            })),
        }
    }

    // pub fn request_memtable_flush(&self, memtable: Arc<Memtable>) -> Result<(), KVRuntimeError> {
    //     let state = self.state.load();

    //     if let Some(sender) = state.memtable_flush_queue().sender() {
    //         sender.send(KVMemtableFlushRequest { memtable }).unwrap();

    //         Ok(())
    //     } else {
    //         Err(KVRuntimeError::with_msg(
    //             KVRuntimeErrorKind::OperationNotAllowed,
    //             "request memtable flush failed - no sender",
    //         ))
    //     }
    // }

    pub fn get_memtable_flush_queue_iter(&self) -> impl Iterator<Item = KVMemtableFlushRequest> {
        let state = self.state.load();

        let receiver = state.memtable_flush_queue().receiver().clone();
        Receiver::into_iter(receiver)
    }

    pub fn new_memtable(&self, table: Arc<Table>) -> Arc<Memtable> {
        let object_id = self.object_id_generator.next();
        let memtable_size = self.config.memtable_size;

        println!(
            "creating new memtable[object_id={}] for table {}[table_id={}]",
            object_id,
            table.table_name,
            table.table_id.as_ref()
        );
        Arc::new(Memtable::new(object_id, table, memtable_size))
    }

    pub fn state_snapshot(&self) -> KVInstanceStateSnapshot {
        KVInstanceStateSnapshot {
            object_id_generator_state: self.object_id_generator.state(),
        }
    }

    pub fn close(&self) {
        self.state.mutate_blocking(|state| {
            let KVInstanceState::Active {
                memtable_flush_queue,
            } = state
            else {
                println!("instance is not in active state - operation aborted");
                return None;
            };

            println!("closing kv instance");

            let mut memtable_flush_queue = memtable_flush_queue.clone();
            memtable_flush_queue.close();

            let next_state = KVInstanceState::Closed {
                memtable_flush_queue,
            };

            Some(next_state)
        });
    }

    pub fn generate_compacted_sstable_id(&self) -> ObjectId {
        self.object_id_generator.next()
    }
}

#[derive(Debug)]
pub struct KVInstanceStateSnapshot {
    pub object_id_generator_state: u64,
}

impl KVInstanceStateSnapshot {
    pub fn new() -> Self {
        Self {
            object_id_generator_state: 0,
        }
    }
}

#[derive(Debug)]
pub enum KVInstanceState {
    Active {
        memtable_flush_queue: MemtableFlushQueue,
    },
    Closed {
        memtable_flush_queue: MemtableFlushQueue,
    },
}

impl KVInstanceState {
    pub fn memtable_flush_queue(&self) -> &MemtableFlushQueue {
        match self {
            Self::Active {
                memtable_flush_queue,
            } => memtable_flush_queue,
            Self::Closed {
                memtable_flush_queue,
            } => memtable_flush_queue,
        }
    }
}
