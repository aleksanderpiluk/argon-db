use std::sync::Arc;

use crate::kv::{memtable::Memtable, primary_key::PrimaryKeySchema, schema::KVColumnsSchema};

pub struct MemtableFactory;

impl MemtableFactory {
    pub fn new(columns_schema: &KVColumnsSchema) -> Arc<Memtable> {
        let primary_key_schema = Arc::new(PrimaryKeySchema::from_columns_schema(columns_schema));
        Arc::new(Memtable::new(primary_key_schema, MEMTABLE_SIZE))
    }
}

const MEMTABLE_SIZE: usize = 1024;
