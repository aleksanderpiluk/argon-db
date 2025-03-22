use std::sync::OnceLock;

use commitlog::CommitLog;
use communication::Channels;
use dashmap::DashMap;
use keyspace::Keyspace;
use mvcc::MVCC;
use operations::OperationsStore;

mod commitlog;
mod communication;
mod data_types;
mod keyspace;
mod memstore;
mod mvcc;
pub mod operations;
pub mod routines;
mod table;

pub struct Database {
    keyspaces: DashMap<String, Keyspace>,
    mvcc: MVCC,
    commitlog: CommitLog,
}

impl Database {
    pub fn channels() -> &'static Channels {
        static CHANNELS: OnceLock<Channels> = OnceLock::new();
        CHANNELS.get_or_init(|| Channels::default())
    }

    pub fn operations_store() -> &'static OperationsStore {
        static OPERATIONS_STORE: OnceLock<OperationsStore> = OnceLock::new();
        OPERATIONS_STORE.get_or_init(|| OperationsStore::default())
    }
}
