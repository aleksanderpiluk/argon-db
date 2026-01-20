mod flush_pre_stats;
mod flush_request;
mod lock;

pub use flush_pre_stats::KVFlushPreStats;
pub use flush_request::KVMemtableFlushRequest;

use crate::kv::{
    KVColumnFilter, KVRangeScanResult, KVRuntimeErrorKind, KVSSTableBuilder, KVScanIterator,
    KVScanIteratorItem, KVTable,
    error::KVRuntimeError,
    iter::PrintIter,
    memtable::lock::MemtableLock,
    mutation::{KVMutation, MutationComparator, MutationUtils, StructuredMutation},
    object_id::ObjectId,
    primary_key::{KVPrimaryKeyMarker, KVPrimaryKeySchema},
    scan::{KVRangeScan, KVScannable},
};
use async_trait::async_trait;
use crossbeam_skiplist::{SkipSet, set::Entry};
use std::{
    mem::replace,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug)]
pub struct Memtable {
    pub object_id: ObjectId,
    table: Arc<KVTable>,
    primary_key_schema: Arc<KVPrimaryKeySchema>,
    inner: SkipSet<MemtableMutation>,
    size_limit: usize,
    size: AtomicUsize,
    lock: MemtableLock,
}

impl Memtable {
    pub fn new(object_id: ObjectId, table: Arc<KVTable>, size_limit: usize) -> Self {
        let primary_key_schema =
            Arc::new(KVPrimaryKeySchema::from_columns_schema(&table.table_schema));

        Self {
            object_id,
            table,
            primary_key_schema,
            inner: SkipSet::new(),
            size_limit,
            size: AtomicUsize::new(0),
            lock: MemtableLock::new(),
        }
    }

    pub fn table(&self) -> &Arc<KVTable> {
        &self.table
    }

    pub fn as_scannable(&self) -> &dyn KVScannable {
        self
    }

    pub fn insert_mutation(
        &self,
        mutation: &StructuredMutation,
    ) -> Result<(), MemtableInsertError> {
        assert!(!MutationUtils::is_marker(mutation));

        self.lock
            .obtain_write_access()
            .map_err(|_| MemtableInsertError::ReadOnlyMode)?;

        let mutation_size = mutation.size();

        loop {
            let memtable_size = self.size.load(Ordering::Acquire);
            let new_memtable_size = memtable_size + mutation_size;

            if new_memtable_size > self.size_limit {
                self.lock.enable_read_only_mode();
                self.lock.release_write_access();
                return Err(MemtableInsertError::SizeExceeded);
            }

            if let Ok(_) = self.size.compare_exchange(
                memtable_size,
                new_memtable_size,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                self.inner.insert(MemtableMutation {
                    primary_key_schema: self.primary_key_schema.clone(),
                    mutation: mutation.clone(),
                });

                self.lock.release_write_access();

                #[cfg(debug_assertions)]
                println!(
                    "[Memtable id: {}] inserted mutation {}",
                    self.object_id,
                    MutationUtils::debug_fmt(&self.table.table_schema, mutation).unwrap()
                );
                return Ok(());
            }
        }
    }

    pub fn is_flush_needed(&self) -> bool {
        self.size.load(Ordering::SeqCst) > 0
    }

    pub fn is_read_only(&self) -> bool {
        self.lock.is_read_only()
    }

    pub fn is_flush_ready(&self) -> bool {
        self.lock.is_flush_ready()
    }

    fn get_range_iterator<'a>(
        &'a self,
        scan: &KVRangeScan,
    ) -> Result<MemtableScanResultsIterInner<'a>, KVRuntimeError> {
        let from = scan.from();
        let to = scan.to();

        match (from, to) {
            (KVPrimaryKeyMarker::Start, KVPrimaryKeyMarker::End) => {
                Ok(Box::new(self.inner.range(..).into_iter()))
            }
            (KVPrimaryKeyMarker::Start, KVPrimaryKeyMarker::Key(pk)) => Ok(Box::new(
                self.inner
                    .range(..MemtableMutation::end(self.primary_key_schema.clone(), pk.clone()))
                    .into_iter(),
            )),
            (KVPrimaryKeyMarker::Key(pk), KVPrimaryKeyMarker::End) => Ok(Box::new(
                self.inner
                    .range(MemtableMutation::start(self.primary_key_schema.clone(), pk.clone())..)
                    .into_iter(),
            )),
            (KVPrimaryKeyMarker::Key(pk_from), KVPrimaryKeyMarker::Key(pk_to)) => Ok(Box::new(
                self.inner
                    .range(
                        MemtableMutation::start(self.primary_key_schema.clone(), pk_from.clone())
                            ..MemtableMutation::end(self.primary_key_schema.clone(), pk_to.clone()),
                    )
                    .into_iter(),
            )),
            _ => {
                return Err(KVRuntimeError::with_msg(
                    KVRuntimeErrorKind::DataMalformed,
                    "invalid scan range",
                ));
            }
        }
    }

    pub async fn flush(
        &self,
        sstable_builder: &mut dyn KVSSTableBuilder,
    ) -> Result<(), KVRuntimeError> {
        if !self.is_read_only() {
            return Err(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                "flush failed - not in read only state",
            ));
        }

        if self.size.load(Ordering::SeqCst) == 0 {
            return Err(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                "flush failed - memtable empty",
            ));
        }

        let scan = self
            .range_scan(&KVRangeScan::new(
                self.table.table_schema.clone(),
                KVPrimaryKeyMarker::Start,
                KVPrimaryKeyMarker::End,
                KVColumnFilter::All,
            ))
            .await?;

        let KVRangeScanResult::Iter(mut iter) = scan else {
            return Err(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                "flush failed - failed to obrain scan iter",
            ));
        };

        while let Some(item) = iter.next_mutation().await {
            sstable_builder.add_mutation(item.mutation()).await.unwrap();
        }

        Ok(())
    }

    pub fn request_flush(self: &Arc<Self>) -> Result<(), KVRuntimeError> {
        println!(
            "requesting flush for memtable[object_id={}]",
            self.object_id
        );

        self.lock.enable_read_only_mode();
        self.table.instance.request_memtable_flush(self.clone())?;

        Ok(())
    }

    pub fn get_flush_prestats(&self) -> Result<KVFlushPreStats, KVRuntimeError> {
        Ok(KVFlushPreStats {
            mutations_count: self.inner.len(),
        })
    }
}

impl std::fmt::Display for Memtable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memtable(object_id={})", self.object_id)
    }
}

#[async_trait]
impl KVScannable for Memtable {
    async fn range_scan(&self, scan: &KVRangeScan) -> Result<KVRangeScanResult, KVRuntimeError> {
        let iter = self.get_range_iterator(scan)?;
        Ok(KVRangeScanResult::Iter(Box::new(PrintIter::new(
            format!("Memtable id={}", self.object_id),
            MemtableScanResultsIter::new(iter),
            self.table.table_schema.clone(),
        ))))
    }

    async fn row_scan(&self, primary_key: &[u8]) -> Result<KVRangeScanResult, KVRuntimeError> {
        let iter = self.get_range_iterator(&KVRangeScan::new(
            self.table.table_schema.clone(),
            KVPrimaryKeyMarker::Key(primary_key.to_vec().into_boxed_slice()),
            KVPrimaryKeyMarker::Key(primary_key.to_vec().into_boxed_slice()),
            KVColumnFilter::All,
        ))?;
        Ok(KVRangeScanResult::Iter(Box::new(PrintIter::new(
            format!("Memtable id={}", self.object_id),
            MemtableScanResultsIter::new(iter),
            self.table.table_schema.clone(),
        ))))
    }
}

#[derive(Debug)]
pub enum MemtableInsertError {
    ReadOnlyMode,
    SizeExceeded,
}

#[derive(Debug)]
struct MemtableMutation {
    primary_key_schema: Arc<KVPrimaryKeySchema>,
    mutation: StructuredMutation,
}

impl MemtableMutation {
    fn start(primary_key_schema: Arc<KVPrimaryKeySchema>, primary_key: Box<[u8]>) -> Self {
        Self {
            primary_key_schema,
            mutation: StructuredMutation::start(primary_key).unwrap(),
        }
    }

    fn end(primary_key_schema: Arc<KVPrimaryKeySchema>, primary_key: Box<[u8]>) -> Self {
        Self {
            primary_key_schema,
            mutation: StructuredMutation::end(primary_key).unwrap(),
        }
    }

    fn as_mutation(&self) -> &dyn KVMutation {
        &self.mutation
    }
}

impl Eq for MemtableMutation {}

impl PartialEq for MemtableMutation {
    fn eq(&self, other: &Self) -> bool {
        assert!(Arc::ptr_eq(
            &self.primary_key_schema,
            &other.primary_key_schema
        ));

        MutationComparator::eq(&self.primary_key_schema, &self.mutation, &other.mutation).unwrap()
    }
}

impl Ord for MemtableMutation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        assert!(Arc::ptr_eq(
            &self.primary_key_schema,
            &other.primary_key_schema
        ));

        MutationComparator::cmp(&self.primary_key_schema, &self.mutation, &other.mutation).unwrap()
    }
}

impl PartialOrd for MemtableMutation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

type MemtableScanResultsIterInner<'a> =
    Box<dyn Iterator<Item = Entry<'a, MemtableMutation>> + Send + Sync + 'a>;

struct MemtableScanResultsIter {
    // mutations: Vec<Box<dyn KVScanIteratorItem + Send + Sync>>,
    current_item: Option<Box<dyn KVScanIteratorItem + Send + Sync>>,
    inner_iter: std::vec::IntoIter<Box<dyn KVScanIteratorItem + Send + Sync>>,
}

impl MemtableScanResultsIter {
    fn new<'a>(iter: MemtableScanResultsIterInner<'a>) -> Self {
        let mut mutations: Vec<Box<dyn KVScanIteratorItem + Send + Sync>> = vec![];

        for entry in iter {
            mutations.push(Box::new(MemtableScanResultsIterItem {
                mutation: entry.mutation.clone(),
            }));
        }

        let mut inner_iter = mutations.into_iter();
        let current_item = inner_iter.next();

        Self {
            current_item,
            inner_iter,
        }
    }
}

#[async_trait]
impl KVScanIterator for MemtableScanResultsIter {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        if self.current_item.is_none() {
            return None;
        }

        let next_item = self.inner_iter.next();
        replace(&mut self.current_item, next_item)
    }

    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>> {
        self.current_item.as_ref()
    }
}

struct MemtableScanResultsIterItem {
    mutation: StructuredMutation,
}

impl KVScanIteratorItem for MemtableScanResultsIterItem {
    fn mutation(&self) -> &(dyn KVMutation + Send + Sync) {
        MutationUtils::as_dyn(&self.mutation)
    }

    fn primary_key(&self) -> &[u8] {
        self.mutation.primary_key()
    }
}
