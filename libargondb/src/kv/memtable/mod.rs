mod lock;

use crate::kv::{
    error::KVRuntimeError,
    memtable::lock::MemtableLock,
    mutation::{Mutation, MutationComparator, MutationUtils, StructuredMutation},
    primary_key::{KVPrimaryKeySchema, PrimaryKeyMarker},
    scan::{KVRangeScan, KVScanIterator, KVScannable},
};
use crossbeam_skiplist::{SkipSet, set::Entry};
use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug)]
pub struct Memtable {
    primary_key_schema: Arc<KVPrimaryKeySchema>,
    inner: SkipSet<MemtableMutation>,
    size_limit: usize,
    size: AtomicUsize,
    lock: MemtableLock,
}

impl Memtable {
    pub fn new(primary_key_schema: Arc<KVPrimaryKeySchema>, size_limit: usize) -> Self {
        Self {
            primary_key_schema,
            inner: SkipSet::new(),
            size_limit,
            size: AtomicUsize::new(0),
            lock: MemtableLock::new(),
        }
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
                return Ok(());
            }
        }
    }

    pub fn is_read_only(&self) -> bool {
        todo!()
    }

    fn get_range_iterator<'a>(
        &'a self,
        scan: &KVRangeScan,
    ) -> Result<Box<dyn Iterator<Item = Entry<'a, MemtableMutation>> + 'a>, KVRuntimeError> {
        let from = scan.from();
        let to = scan.to();

        match (from, to) {
            (PrimaryKeyMarker::Start, PrimaryKeyMarker::End) => {
                Ok(Box::new(self.inner.range(..).into_iter()))
            }
            (PrimaryKeyMarker::Start, PrimaryKeyMarker::Key(pk)) => Ok(Box::new(
                self.inner
                    .range(..MemtableMutation::end(self.primary_key_schema.clone(), pk.clone()))
                    .into_iter(),
            )),
            (PrimaryKeyMarker::Key(pk), PrimaryKeyMarker::End) => Ok(Box::new(
                self.inner
                    .range(MemtableMutation::start(self.primary_key_schema.clone(), pk.clone())..)
                    .into_iter(),
            )),
            (PrimaryKeyMarker::Key(pk_from), PrimaryKeyMarker::Key(pk_to)) => Ok(Box::new(
                self.inner
                    .range(
                        MemtableMutation::start(self.primary_key_schema.clone(), pk_from.clone())
                            ..MemtableMutation::end(self.primary_key_schema.clone(), pk_to.clone()),
                    )
                    .into_iter(),
            )),
            _ => return Err(KVRuntimeError::DataMalformed),
        }
    }
}

impl KVScannable for Memtable {
    fn range_scan(&self, scan: &KVRangeScan) -> Result<Box<dyn KVScanIterator>, KVRuntimeError> {
        let iter = self.get_range_iterator(scan)?;
        todo!()
    }
}

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

    fn as_mutation(&self) -> &dyn Mutation {
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

// type MemtableScanResultsIterInner<'a> = Box<dyn Iterator<Item = Entry<'a, MemtableMutation>> + 'a>;

// impl KVScanIterator for MemtableScanResultsIter<'_> {
//     fn next_mutation(&mut self) -> Option<&dyn Mutation> {
//         self.current_mutation = Self::iter_next_mutation(&mut self.iter);

//         match &self.current_mutation {
//             Some(mutation) => Some(MutationUtils::as_dyn(mutation)),
//             None => None,
//         }
//     }

//     // fn peek_mutation(&self) -> Option<&dyn Mutation> {
//     //     match &self.current_mutation {
//     //         Some(mutation) => Some(MutationUtils::as_dyn(mutation)),
//     //         None => None,
//     //     }
//     // }
// }
