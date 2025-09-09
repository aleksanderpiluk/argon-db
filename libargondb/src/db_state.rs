use std::sync::Arc;

use crate::memtable::Memtable;

struct StateSnapshot {
    tables: Box<[TableSnapshot]>,
}

impl StateSnapshot {
    fn table_lookup_by_name(&self, table_name: String) -> Option<&TableSnapshot> {
        match self
            .tables
            .binary_search_by(|table| table.table_name.cmp(&table_name))
        {
            Ok(idx) => Some(&self.tables[idx]),
            Err(_) => None,
        }
    }
}

struct TableSnapshot {
    table_name: String,
    schema: (),
    current_memtable: Arc<Memtable>,
}

impl TableSnapshot {
    fn table_name(&self) -> &String {
        &self.table_name
    }

    fn current_memtable(&self) -> &Arc<Memtable> {
        &self.current_memtable
    }
}

struct TableSchemaSnapshot {
    primary_key: (),
    columns: Box<[TableColumnSchema]>,
}

struct TableColumnSchema {
    column_id: u16,
    column_name: String,
}
