use std::sync::Arc;

use super::catalog_state::CatalogState;
use crate::{kv::KVTable, utils::rcu::RCU};

pub struct Catalog {
    state: RCU<CatalogState>,
}

impl Catalog {
    pub fn empty() -> Self {
        Self {
            state: RCU::new(Arc::new(CatalogState::empty())),
        }
    }

    pub fn add_table(&self, table_name: String, table: Arc<KVTable>) {
        self.state.mutate_blocking(move |current_state| {
            let new_state = current_state.add_table(table_name, table);

            Some(new_state)
        });
    }

    pub fn lookup_table_by_name(&self, table_name: &str) -> Option<Arc<KVTable>> {
        let catalog_state = self.state.load();

        catalog_state.lookup_table_by_name(table_name)
    }
}
