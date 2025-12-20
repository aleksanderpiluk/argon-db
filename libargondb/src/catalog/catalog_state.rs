use std::{collections::BTreeMap, sync::Arc};

use crate::kv::{KVTable, KVTableName};

pub struct CatalogState {
    tables: Vec<Arc<KVTable>>,
    table_name_map: BTreeMap<KVTableName<'static>, Arc<KVTable>>,
}

impl CatalogState {
    pub fn empty() -> Self {
        Self {
            tables: Vec::new(),
            table_name_map: BTreeMap::new(),
        }
    }

    pub fn add_table(&self, table: Arc<KVTable>) -> Self {
        let mut tables = self.tables.clone();
        tables.push(table.clone());

        let mut table_name_map = self.table_name_map.clone();
        table_name_map.insert(table.table_name.clone(), table.clone());

        Self {
            tables,
            table_name_map,
        }
    }

    pub fn list_tables(&self) -> Vec<Arc<KVTable>> {
        self.tables.clone()
    }

    pub fn lookup_table_by_name(&self, table_name: &KVTableName) -> Option<Arc<KVTable>> {
        self.table_name_map
            .get(table_name)
            .map(|table_ref| table_ref.clone())
    }
}
