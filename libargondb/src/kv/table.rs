use std::{ops::Deref, sync::Arc};

use crate::{
    kv::{
        KVSSTable, KVScanExecutor,
        config::KVConfig,
        factory::{self, KVFactory},
        mutation::StructuredMutation,
        scan::KVScanOp,
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
    pub fn create(
        config: KVConfig,
        columns_schema: KVTableSchema,
        sstables: Vec<Arc<KVSSTable>>,
    ) -> Self {
        let factory = KVFactory::new(config);

        let memtable = factory.new_memtable(&columns_schema);
        let table_state = KVTableState::for_new_table(columns_schema, memtable, sstables);

        Self {
            factory,
            state: RCU::new(Arc::new(table_state)),
        }
    }

    pub fn insert_mutations(&self, mutations: &Vec<StructuredMutation>) {
        todo!()
    }

    pub fn scan(&self, scan_op: impl KVScanOp) {
        let table_state = self.state.load();
        let result = KVScanExecutor::execute(&table_state, scan_op).unwrap();
        todo!()
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

    // pub fn load_state(&self) -> impl Deref<Target = Arc<KVTableState>> {
    //     self.state.load()
    // }

    // pub async fn add_sstables(&self, sstables: Vec<Arc<()>>) {
    //     self.state
    //         .mutate(|state| {
    //             let mut next_state = state.clone();

    //             next_state.sstables.extend(sstables);

    //             Some(next_state)
    //         })
    //         .await;
    // }
}
