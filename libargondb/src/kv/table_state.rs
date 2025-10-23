use std::sync::Arc;

use crate::kv::{memtable::Memtable, scan::KVScannable, schema::KVTableSchema};

#[derive(Clone, Debug)]
pub struct KVTableState {
    flush_queue: (),
    pub columns_schema: KVTableSchema,
    pub read_memtables: Vec<Arc<Memtable>>,
    pub current_memtable: Arc<Memtable>,
}

impl KVTableState {
    pub fn for_new_table(columns_schema: KVTableSchema, memtable: Arc<Memtable>) -> Self {
        Self {
            flush_queue: (),
            columns_schema,
            read_memtables: vec![],
            current_memtable: memtable,
        }
    }

    pub fn list_scannable(&self) -> Vec<&dyn KVScannable> {
        let mut scannable: Vec<&dyn KVScannable> = vec![];

        scannable.push(self.current_memtable.as_scannable());

        for memtable in &self.read_memtables {
            scannable.push(memtable.as_scannable());
        }

        // TODO: SStables

        scannable
    }
}
