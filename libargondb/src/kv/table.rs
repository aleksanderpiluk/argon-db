use std::{ops::Deref, sync::Arc};

use crate::{
    kv::{
        config::KVConfig,
        factory::{self, KVFactory},
        schema::KVTableSchema,
        table_state::KVTableState,
    },
    utils::rcu::RCU,
};

#[derive(Debug)]
pub struct KVTable {
    factory: KVFactory,
    state: RCU<KVTableState>,
}

impl KVTable {
    pub fn create(config: KVConfig, columns_schema: KVTableSchema) -> Self {
        let factory = KVFactory::new(config);

        let memtable = factory.new_memtable(&columns_schema);
        let table_state = KVTableState::for_new_table(columns_schema, memtable);

        Self {
            factory,
            state: RCU::new(Arc::new(table_state)),
        }
    }

    pub fn load_state(&self) -> impl Deref<Target = Arc<KVTableState>> {
        self.state.load()
    }

    pub async fn flush_current_memtable(&self) {
        self.state
            .mutate(|state| {
                let mut next_state: KVTableState = state.clone();

                let current_memtable = next_state.current_memtable;
                if !current_memtable.is_read_only() {
                    return None;
                }

                next_state.current_memtable = self.factory.new_memtable(&next_state.columns_schema);
                next_state.read_memtables.push(current_memtable);

                // todo!("add to flush queue or sth");

                Some(next_state)
            })
            .await;
    }
}
