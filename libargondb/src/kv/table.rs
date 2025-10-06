use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use arc_swap::ArcSwap;

use crate::kv::{
    column_type::{self, ColumnType},
    memtable::Memtable,
};

pub struct Table {
    state_mut_lock: Mutex<()>, // TODO: CHANGE LOCK
    state: ArcSwap<TableState>,
}

struct TableState {
    flush_queue: (),
    columns_schema: ColumnsSchema,
    read_memtables: Vec<Arc<Memtable>>,
    current_memtable: Arc<Memtable>,
}

impl Table {
    // pub fn columns_schema(&self) -> &ColumnsSchema {
    // }

    pub fn insert_mutation(&self) {
        let current_membtable = self.state.load().current_memtable.clone();

        todo!()
    }

    pub fn range_scan(&self) {}

    fn flush_current_memtable(&self) {
        let guard = self.state_mut_lock.lock().unwrap();

        let state = self.state.load_full();
        let current_memtable = state.current_memtable;
        todo!("check/enable read-only");
        state.current_memtable = Memtable::new();
        state.read_memtables.push(current_memtable);
        self.state.store(state);

        drop(guard);
    }
}

pub struct ColumnsSchema {
    columns_map: BTreeMap<u16, ColumnSchema>,
}

impl ColumnsSchema {
    pub fn columns_count(&self) -> u16 {
        let len = self.columns_map.len();
        assert!(len <= u16::MAX as usize);

        len as u16
    }

    pub fn column_schema(&self, column_id: u16) -> Option<&ColumnSchema> {
        self.columns_map.get(&column_id)
    }
}

pub struct ColumnSchema {}

impl ColumnSchema {
    pub fn column_type(&self) -> impl ColumnType {
        todo!();
        column_type::Bytes
    }

    pub fn column_name(&self) -> &str {
        todo!()
    }
}
