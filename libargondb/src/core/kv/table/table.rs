use super::{KVTableId, KVTableName, KVTableState};
use crate::{
    kv::{
        KVRangeScanResult, KVRuntimeError, KVRuntimeErrorKind, KVSSTable, KVScannable,
        instance::KVInstance,
        iter::{PrintIter, ShadowingIter},
        memtable::{Memtable, MemtableInsertError},
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
        sstables: Vec<Box<dyn KVSSTable>>,
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

    pub async fn insert_mutations(
        self: &Arc<Self>,
        mutations: &Vec<StructuredMutation>,
    ) -> Result<(), KVRuntimeError> {
        let mut state = self.state.load();
        let mut active_state = state.try_as_active()?;

        for mutation in mutations {
            let mut attempt_count = 0u8;
            let mut success = false;
            let max_attempts_count = 3u8;

            while !success && attempt_count < max_attempts_count {
                attempt_count += 1;

                let insert_result = active_state.current_memtable.insert_mutation(mutation);

                if let Err(err) = insert_result {
                    match err {
                        MemtableInsertError::ReadOnlyMode | MemtableInsertError::SizeExceeded => {
                            self.request_current_memtable_flush().await;

                            state = self.state.load();
                            active_state = state.try_as_active()?;
                        }
                    }
                } else {
                    success = true;
                    break;
                }
            }

            if !success {
                return Err(KVRuntimeError::with_msg(
                    KVRuntimeErrorKind::OperationFailure,
                    "insert failed - max attempt count exceeded",
                ));
            }
        }

        Ok(())
    }

    pub async fn scan(&self, scan_op: impl KVScanOp) -> Result<KVRowIter, KVRuntimeError> {
        #[cfg(debug_assertions)]
        println!("table scan op: {}", scan_op);

        let table_state = self.state.load();

        let pk_schema = KVPrimaryKeySchema::from_table_schema(&self.table_schema);
        let mut result_iter = KVMergeScanIter::new(pk_schema.clone());

        for scannable in table_state.list_scannable()? {
            let scan_result = scan_op.scan(scannable).await?;
            if let KVRangeScanResult::Iter(scannable_iter) = scan_result {
                #[cfg(debug_assertions)]
                println!("result_iter add scannable: {}", scannable);
                result_iter.add_iter(scannable_iter);
            } else {
                #[cfg(debug_assertions)]
                println!("result_iter reject scannable: {}", scannable);
            }
        }

        let scan_iter = Box::new(PrintIter::new(
            "Final",
            ShadowingIter::new(result_iter, pk_schema).await,
            self.table_schema.clone(),
        ));

        Ok(KVRowIter::new(self.table_schema.clone(), scan_iter))
    }

    pub async fn request_current_memtable_flush(self: &Arc<Self>) {
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
        sstable: Arc<Box<dyn KVSSTable>>,
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

    pub fn list_sstables(&self) -> Vec<Arc<Box<dyn KVSSTable>>> {
        let state = self.state.load();

        state.list_sstables()
    }

    pub fn replace_compacted_sstables(
        &self,
        compacted_sstables: &Vec<Arc<Box<dyn KVSSTable>>>,
        new_sstable: Arc<Box<dyn KVSSTable>>,
    ) -> Result<(), ()> {
        let state_replaced = self.state.mutate_blocking(|state| {
            match state.replace_compacted_sstables(compacted_sstables, new_sstable) {
                Err(e) => {
                    println!("Failed to replace flushed memtable with sstable: {:?}", e);
                    None
                }
                Ok(next_state) => Some(next_state),
            }
        });

        if state_replaced { Ok(()) } else { Err(()) }
    }
}
