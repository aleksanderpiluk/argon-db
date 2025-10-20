use std::sync::Arc;

use crate::kv::{
    column_type::{ColumnType, ColumnTypeCode},
    memtable::Memtable,
    memtable_factory::MemtableFactory,
    schema::KVColumnsSchema,
};

#[derive(Clone, Debug)]
pub struct KVTableState {
    flush_queue: (),
    pub columns_schema: KVColumnsSchema,
    pub read_memtables: Vec<Arc<Memtable>>,
    pub current_memtable: Arc<Memtable>,
}

impl KVTableState {
    pub fn for_new_table(columns_schema: KVColumnsSchema) -> Self {
        let memtable = MemtableFactory::new(&columns_schema);

        Self {
            flush_queue: (),
            columns_schema,
            read_memtables: vec![],
            current_memtable: memtable,
        }
    }
}
