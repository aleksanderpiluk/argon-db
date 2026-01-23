use std::sync::Arc;

use crate::kv::memtable::Memtable;

pub struct KVMemtableFlushRequest {
    pub memtable: Arc<Memtable>,
}
