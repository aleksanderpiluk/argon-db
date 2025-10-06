use crate::kv::{mutation::Mutation, primary_key::PrimaryKeySchema};

pub trait Scannable {
    fn range_scan(&self, scan: RangeScanParams) -> impl ScanResultIter;
    fn set_scan(&self, scan: SetScanParams) -> impl ScanResultIter;
}

pub struct RangeScanParams {
    from: Box<[u8]>,
    to: Box<[u8]>,
    columns: ColumnFilter,
}

impl RangeScanParams {
    pub fn new(from: Box<[u8]>, to: Box<[u8]>, columns: ColumnFilter) -> Self {
        assert!(from <= to);

        Self { from, to, columns }
    }

    pub fn from(&self) -> &[u8] {
        &self.from
    }

    pub fn to(&self) -> &[u8] {
        &self.to
    }
}

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
    heap: Box<[Box<dyn ScanResultIter>]>,
}

impl<'a> ScanResultItersMerge<'a> {
    pub fn new(
        primary_key_schema: &'a PrimaryKeySchema,
        iters: Vec<Box<dyn ScanResultIter>>,
    ) -> Self {
        let heap = iters.into_boxed_slice();

        Self {
            primary_key_schema,
            heap,
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
