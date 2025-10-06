use crate::kv::{
    mutation::{Mutation, MutationComparator, MutationUtils, StructuredMutation},
    primary_key::PrimaryKeySchema,
    scan::{RangeScanParams, ScanResultIter, Scannable},
};
use crossbeam_skiplist::SkipSet;
use std::{
    ops::Range,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

pub struct Memtable {
    primary_key_schema: Arc<PrimaryKeySchema>,
    inner: SkipSet<MemtableMutation>,
    size_limit: usize,
    size: AtomicUsize,
    state: MemtableState,
}

impl Memtable {
    pub fn insert_mutation(&self, mutation: StructuredMutation) -> Result<(), MemtableInsertError> {
        assert!(!MutationUtils::is_marker(&mutation));

        self.state
            .try_write_access()
            .map_err(|_| MemtableInsertError::ReadOnlyMode)?;

        let mutation_size = mutation.size();

        loop {
            let memtable_size = self.size.load(Ordering::Acquire);
            let new_memtable_size = memtable_size + mutation_size;

            if new_memtable_size > self.size_limit {
                self.state.release_write_access();
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
                    mutation,
                });

                self.state.release_write_access();
                return Ok(());
            }
        }
    }
}

impl Scannable for Memtable {
    fn range_scan(&self, scan: RangeScanParams) -> impl ScanResultIter {
        let from = MemtableMutation::start(self.primary_key_schema.clone(), Box::from(scan.from()));
        let to = MemtableMutation::end(self.primary_key_schema.clone(), Box::from(scan.to()));

        let range = self.inner.range(from..to);

        MemtableScanResultsIter::new(range)
    }

    fn set_scan(&self, scan: super::scan::SetScanParams) -> MemtableScanResultsIter {
        todo!()
    }
}

enum MemtableInsertError {
    ReadOnlyMode,
    SizeExceeded,
}

struct MemtableState(AtomicUsize);

impl MemtableState {
    const READ_ONLY_FLAG: usize = 1 << 63;

    fn enable_read_only_mode(&self) {
        self.0.fetch_xor(Self::READ_ONLY_FLAG, Ordering::AcqRel);
    }

    fn try_write_access(&self) -> Result<(), ()> {
        loop {
            let state = self.0.load(Ordering::Acquire);

            let is_read_only = (state & Self::READ_ONLY_FLAG) > 0;
            if is_read_only {
                return Err(());
            }

            // TODO: Add debug assertions
            let new_state = state + 1;

            if let Ok(_) =
                self.0
                    .compare_exchange(state, new_state, Ordering::Release, Ordering::Relaxed)
            {
                return Ok(());
            }
        }
    }

    fn release_write_access(&self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

struct MemtableMutation {
    primary_key_schema: Arc<PrimaryKeySchema>,
    mutation: StructuredMutation,
}

impl MemtableMutation {
    fn start(primary_key_schema: Arc<PrimaryKeySchema>, primary_key: Box<[u8]>) -> Self {
        Self {
            primary_key_schema,
            mutation: StructuredMutation::start(primary_key).unwrap(),
        }
    }

    fn end(primary_key_schema: Arc<PrimaryKeySchema>, primary_key: Box<[u8]>) -> Self {
        Self {
            primary_key_schema,
            mutation: StructuredMutation::end(primary_key).unwrap(),
        }
    }
}

impl Eq for MemtableMutation {}

impl PartialEq for MemtableMutation {
    fn eq(&self, other: &Self) -> bool {
        assert!(Arc::ptr_eq(
            &self.primary_key_schema,
            &other.primary_key_schema
        ));

        MutationComparator::eq(&self.primary_key_schema, &self.mutation, &other.mutation)
    }
}

impl Ord for MemtableMutation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        assert!(Arc::ptr_eq(
            &self.primary_key_schema,
            &other.primary_key_schema
        ));

        MutationComparator::cmp(&self.primary_key_schema, &self.mutation, &other.mutation)
    }
}

impl PartialOrd for MemtableMutation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

type MemtableScanResultsIterInner<'a> =
    crossbeam_skiplist::set::Range<'a, MemtableMutation, Range<MemtableMutation>, MemtableMutation>;
struct MemtableScanResultsIter<'a> {
    iter: MemtableScanResultsIterInner<'a>,
    current_mutation: Option<StructuredMutation>,
}

impl<'a> MemtableScanResultsIter<'a> {
    fn new(mut iter: MemtableScanResultsIterInner<'a>) -> Self {
        Self {
            iter,
            current_mutation: None,
        }
    }

    fn iter_next_mutation(
        iter: &mut MemtableScanResultsIterInner<'a>,
    ) -> Option<StructuredMutation> {
        todo!()
    }
}

impl ScanResultIter for MemtableScanResultsIter<'_> {
    fn next_mutation(&mut self) -> Option<&dyn Mutation> {
        self.current_mutation = Self::iter_next_mutation(&mut self.iter);

        match &self.current_mutation {
            Some(mutation) => Some(MutationUtils::as_dyn(mutation)),
            None => None,
        }
    }

    // fn peek_mutation(&self) -> Option<&dyn Mutation> {
    //     match &self.current_mutation {
    //         Some(mutation) => Some(MutationUtils::as_dyn(mutation)),
    //         None => None,
    //     }
    // }
}
