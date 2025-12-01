use std::{collections::BTreeMap, sync::Arc};

use crate::kv::KVTable;

pub struct CatalogState {
    tables: Vec<Arc<KVTable>>,
    table_name_map: BTreeMap<String, Arc<KVTable>>,
}

impl CatalogState {
    pub fn empty() -> Self {
        Self {
            tables: Vec::new(),
            table_name_map: BTreeMap::new(),
        }
    }

    pub fn add_table(&self, table_name: String, table: Arc<KVTable>) -> Self {
        let mut tables = self.tables.clone();
        tables.push(table.clone());

        let mut table_name_map = self.table_name_map.clone();
        table_name_map.insert(table_name, table.clone());

        Self {
            tables,
            table_name_map,
        }
    }

    pub fn lookup_table_by_name(&self, table_name: &str) -> Option<Arc<KVTable>> {
        self.table_name_map
            .get(table_name)
            .map(|table_ref| table_ref.clone())
    }
}
