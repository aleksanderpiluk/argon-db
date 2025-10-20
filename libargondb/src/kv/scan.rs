use std::cmp::Ordering;

use crate::kv::{
    error::KVRuntimeError,
    memtable,
    mutation::{Mutation, StructuredMutation},
    primary_key::{PrimaryKeyComparator, PrimaryKeySchema},
    table_state::KVTableState,
};

pub enum PrimaryKeyMarker {
    Start,
    End,
    Key(Box<[u8]>),
}

struct PrimaryKeyMarkerComparator;

impl PrimaryKeyMarkerComparator {
    pub fn cmp(
        schema: &PrimaryKeySchema,
        this: &PrimaryKeyMarker,
        that: &PrimaryKeyMarker,
    ) -> Result<Ordering, KVRuntimeError> {
        if let PrimaryKeyMarker::Start = this {
            if let PrimaryKeyMarker::Start = that {
                return Ok(Ordering::Equal);
            } else {
                return Ok(Ordering::Less);
            }
        }

        if let PrimaryKeyMarker::End = this {
            if let PrimaryKeyMarker::End = that {
                return Ok(Ordering::Equal);
            } else {
                return Ok(Ordering::Greater);
            }
        }

        if let PrimaryKeyMarker::Key(this_key) = this {
            return match that {
                PrimaryKeyMarker::Key(that_key) => {
                    PrimaryKeyComparator::cmp(schema, &this_key, &that_key)
                }
                PrimaryKeyMarker::Start => Ok(Ordering::Greater),
                PrimaryKeyMarker::End => Ok(Ordering::Less),
            };
        }

        panic!("PrimaryKeyMarkerComparator fatal error");
    }
}

pub trait Scannable {
    fn range_scan(
        &self,
        scan: &RangeScanParams,
    ) -> Result<Box<dyn ScanResultIter + '_>, KVRuntimeError>;
    fn set_scan(&self, scan: SetScanParams) -> impl ScanResultIter;
}

trait ScanParams {}

pub struct RangeScanParams {
    from: PrimaryKeyMarker,
    to: PrimaryKeyMarker,
    columns: ColumnFilter,
}

impl RangeScanParams {
    pub fn new(from: PrimaryKeyMarker, to: PrimaryKeyMarker, columns: ColumnFilter) -> Self {
        Self { from, to, columns }
    }

    pub fn from(&self) -> &PrimaryKeyMarker {
        &self.from
    }

    pub fn to(&self) -> &PrimaryKeyMarker {
        &self.to
    }
}

impl ScanParams for RangeScanParams {}

pub struct SetScanParams {
    rows: Box<[Box<[u8]>]>,
    columns: ColumnFilter,
}

impl SetScanParams {
    pub fn new(rows: Box<[Box<[u8]>]>, columns: ColumnFilter) -> Self {
        assert!(rows.is_sorted()); //TODO: 

        Self { rows, columns }
    }
}

pub enum ColumnFilter {
    All,
}

pub trait ScanResultIter {
    fn next_mutation(&mut self) -> Option<&dyn Mutation>;
    // fn peek_mutation(&self) -> Option<&dyn Mutation>;
}

pub struct ScanResultItersMerge<'a> {
    primary_key_schema: &'a PrimaryKeySchema,
    // heap: Box<[Box<dyn ScanResultIter>]>,
}

impl<'a> ScanResultItersMerge<'a> {
    pub fn new(
        primary_key_schema: &'a PrimaryKeySchema,
        iters: Vec<Box<dyn ScanResultIter + 'a>>,
    ) -> Self {
        // let heap = iters.into_boxed_slice();

        println!("Scan results");
        for mut iter in iters {
            while let Some(mutation) = iter.next_mutation() {
                println!("{:#?}", StructuredMutation::from_mutation(mutation));
            }
        }

        Self {
            primary_key_schema,
            // heap,
        }
    }

    fn heapify(heap: &mut Box<[Box<dyn ScanResultIter>]>) {
        todo!()
    }
}

impl ScanResultIter for ScanResultItersMerge<'_> {
    fn next_mutation(&mut self) -> Option<&dyn Mutation> {
        todo!()
    }
}

pub struct ScanExecutor;

impl ScanExecutor {
    // pub fn execute(table: &KVTableState, scan: impl ScanParams) {
    pub fn execute(table: &KVTableState, scan: RangeScanParams) -> Result<(), KVRuntimeError> {
        let mut result_iters = vec![];

        result_iters.push(table.current_memtable.range_scan(&scan)?);

        for memtable in &table.read_memtables {
            result_iters.push(memtable.range_scan(&scan)?);
        }

        // TODO: SStables

        let pk_schema = PrimaryKeySchema::from_columns_schema(&table.columns_schema);
        ScanResultItersMerge::new(&pk_schema, result_iters);

        todo!()
    }
}

// impl PartialEq for ScanResultItersMergeHeapItem<'_> {
//     fn eq(&self, other: &Self) -> bool {
//         match (self.iter.peek_mutation(), other.iter.peek_mutation()) {
//             (None, None) => true,
//             (Some(this), Some(that)) => MutationComparator::eq(self.primary_key_schema, this, that),
//             _ => false,
//         }
//     }
// }

// impl Eq for ScanResultItersMergeHeapItem<'_> {}

// impl PartialOrd for ScanResultItersMergeHeapItem<'_> {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Ord for ScanResultItersMergeHeapItem<'_> {
//     fn cmp(&self, other: &Self) -> Ordering {
//         match (self.iter.peek_mutation(), other.iter.peek_mutation()) {
//             (None, None) => Ordering::Equal,
//             (Some(_), None) => Ordering::Greater,
//             (None, Some(_)) => Ordering::Less,
//             (Some(this), Some(that)) => {
//                 MutationComparator::cmp(self.primary_key_schema, this, that)
//             }
//         }
//     }
// }

// struct ScanResult(Box<[u8]>);

// impl ScanResult {
//     pub fn iter(&self) -> ScanResultIterator<'_> {
//         ScanResultIterator {
//             data: &self,
//             ptr: 0,
//         }
//     }
// }

// struct ScanResultIterator<'a> {
//     data: &'a ScanResult,
//     ptr: usize,
// }

// impl<'a> Iterator for ScanResultIterator<'a> {
//     type Item = InMemoryMutationView<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.ptr < self.data.0.len() {
//             let item = InMemoryMutationView::try_from(&self.data.0[self.ptr..]).unwrap();
//             self.ptr += item.len();

//             Some(item)
//         } else {
//             None
//         }
//     }
// }

// pub struct ScanResultBuilder(Vec<u8>);

// impl ScanResultBuilder {
//     pub fn new() -> Self {
//         Self(Vec::new())
//     }

//     pub fn write(&mut self, mutation: impl Mutation) {
//         // TODO: Assert mutations are put in order
//         InMemoryMutationView::write(&mut self.0, mutation);
//     }

//     pub fn build(self) -> ScanResult {
//         ScanResult(self.0.into_boxed_slice())
//     }
// }
