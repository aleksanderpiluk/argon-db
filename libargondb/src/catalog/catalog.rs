use std::sync::Arc;

use super::catalog_state::CatalogState;
use crate::{
    kv::{KVTable, KVTableName},
    utils::rcu::RCU,
};

pub struct Catalog {
    state: RCU<CatalogState>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            state: RCU::new(Arc::new(CatalogState::empty())),
        }
    }

    pub fn add_table(&self, table: Arc<KVTable>) {
        self.state.mutate_blocking(move |current_state| {
            let new_state = current_state.add_table(table);

            Some(new_state)
        });
    }

    pub fn lookup_table_by_name(&self, table_name: &KVTableName) -> Option<Arc<KVTable>> {
        let catalog_state = self.state.load();

        catalog_state.lookup_table_by_name(table_name)
    }

    pub fn list_tables(&self) -> Vec<Arc<KVTable>> {
        let catalog_state = self.state.load();

        catalog_state.list_tables()
    }
}
