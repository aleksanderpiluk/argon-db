use super::{KVTableId, KVTableName, KVTableState};
use crate::{
    kv::{
        KVRangeScanResult, KVRuntimeError, KVScannable,
        instance::KVInstance,
        memtable::Memtable,
        mutation::StructuredMutation,
        primary_key::KVPrimaryKeySchema,
        scan::KVScanOp,
        scan_iter::{KVMergeScanIter, KVRowIter},
        schema::KVTableSchema,
    },
    utils::rcu::RCU,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct KVTable {
    pub table_id: KVTableId<'static>,
    pub table_name: KVTableName<'static>,
    pub table_schema: KVTableSchema,

    state: RCU<KVTableState>,
    pub instance: Arc<KVInstance>,
}

impl KVTable {
    pub fn create(
        instance: Arc<KVInstance>,
        table_id: KVTableId<'static>,
        table_name: KVTableName<'static>,
        table_schema: KVTableSchema,
        sstables: Vec<Box<dyn KVScannable>>,
    ) -> Self {
        let sstables = sstables
            .into_iter()
            .map(|sstable| Arc::new(sstable))
            .collect();
        let table_state = KVTableState::new_closed(sstables);

        Self {
            instance,

            table_id,
            table_name,
            table_schema,

            state: RCU::new(Arc::new(table_state)),
        }
    }

    pub fn open(self: &Arc<Self>) {
        self.state.mutate_blocking(|state| {
            let Ok(closed_state) = state.try_as_closed() else {
                println!("table is not in closed state - operation aborted");
                return None;
            };

            let next_memtable = self.instance.new_memtable(self.clone());

            let next_state = closed_state.move_to_active(next_memtable);

            Some(next_state)
        });
    }

    pub fn close(&self) {
        self.state.mutate_blocking(|state| {
            let Ok(active_state) = state.try_as_active() else {
                println!("table is not in active state - operation aborted");
                return None;
            };

            let next_state = active_state.move_to_closed();

            Some(next_state)
        });
    }

    pub fn insert_mutations(
        &self,
        mutations: &Vec<StructuredMutation>,
    ) -> Result<(), KVRuntimeError> {
        let state = self.state.load();

        let active_state = state.try_as_active()?;

        for mutation in mutations {
            active_state
                .current_memtable
                .insert_mutation(mutation)
                .unwrap();
        }

        Ok(())
    }

    pub async fn scan(&self, scan_op: impl KVScanOp) -> Result<KVRowIter, KVRuntimeError> {
        let table_state = self.state.load();

        let pk_schema = KVPrimaryKeySchema::from_columns_schema(&self.table_schema);
        let mut result_iter = KVMergeScanIter::new(pk_schema);

        for scannable in table_state.list_scannable()? {
            // TODO: Scannable check preconditions (bloom filters etc.)

            let scan_result = scan_op.scan(scannable).await?;
            if let KVRangeScanResult::Iter(scannable_iter) = scan_result {
                result_iter.add_iter(scannable_iter);
            }
        }

        let scan_iter = Box::new(result_iter);

        Ok(KVRowIter::new(self.table_schema.clone(), scan_iter))
    }

    pub async fn request_current_memtable_flush(self: Arc<Self>) {
        self.state
            .mutate(|state| {
                let Ok(active_state) = state.try_as_active() else {
                    println!("table is not in active state - operation aborted");
                    return None;
                };

                let needs_flush = active_state.current_memtable.is_flush_needed();
                if !needs_flush {
                    println!("current memtable doesn't need flush - operation aborted");
                    return None;
                }

                let next_memtable = self.instance.new_memtable(self.clone());

                let next_state = active_state.replace_current_memtable(next_memtable);

                Some(next_state)
            })
            .await;
    }

    // pub fn load_state(&self) -> impl Deref<Target = Arc<KVTableState>> {
    //     self.state.load()
    // }

    pub fn replace_flushed_memtable_with_sstable(
        &self,
        memtable: Arc<Memtable>,
        sstable: Arc<Box<dyn KVScannable>>,
    ) {
        self.state.mutate_blocking(|state| {
            match state.replace_flushed_memtable_with_sstable(memtable, sstable) {
                Err(e) => {
                    println!("Failed to replace flushed memtable with sstable: {:?}", e);
                    None
                }
                Ok(next_state) => Some(next_state),
            }
        });
    }
}
