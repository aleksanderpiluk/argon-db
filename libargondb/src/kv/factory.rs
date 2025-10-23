use std::{fmt::Debug, sync::Arc};

use crate::kv::{
    config::KVConfig, memtable::Memtable, primary_key::KVPrimaryKeySchema, schema::KVTableSchema,
};

pub struct KVFactory {
    config: KVConfig,
}

impl KVFactory {
    pub fn new(config: KVConfig) -> Self {
        Self { config }
    }

    pub fn new_memtable(&self, columns_schema: &KVTableSchema) -> Arc<Memtable> {
        let memtable_size = self.config.memtable_size;
        let primary_key_schema = Arc::new(KVPrimaryKeySchema::from_columns_schema(columns_schema));

        Arc::new(Memtable::new(primary_key_schema, memtable_size))
    }
}

impl Debug for KVFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KVFactory").finish()
    }
}
