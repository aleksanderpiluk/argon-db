use std::vec;

use async_trait::async_trait;

use crate::kv::{
    error::KVRuntimeError,
    mutation::{KVMutation, MutationComparator},
    primary_key::{KVPrimaryKeyMarker, KVPrimaryKeySchema},
    table_state::KVTableState,
};

pub trait KVScanOp {
    fn scan<'a, T: KVScannable + ?Sized>(
        &self,
        scannable: &'a T,
    ) -> Result<Box<dyn KVScanIterator + Send + Sync>, KVRuntimeError>;
}

#[async_trait]
pub trait KVScannable {
    async fn range_scan(
        &self,
        scan: &KVRangeScan,
    ) -> Result<Box<dyn KVScanIterator>, KVRuntimeError>;
    // fn set_scan(&self, scan: SetScanParams) -> impl ScanResultIter;
}

#[async_trait]
pub trait KVScanIterator {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>>;
    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>>;
}

pub trait KVScanIteratorItem {
    fn primary_key(&self) -> &[u8];

    fn mutation(&self) -> &dyn KVMutation;
}

pub struct KVRangeScan {
    from: KVPrimaryKeyMarker,
    to: KVPrimaryKeyMarker,
    columns: KVColumnFilter,
}

impl KVRangeScan {
    pub fn new(from: KVPrimaryKeyMarker, to: KVPrimaryKeyMarker, columns: KVColumnFilter) -> Self {
        Self { from, to, columns }
    }

    pub fn from(&self) -> &KVPrimaryKeyMarker {
        &self.from
    }

    pub fn to(&self) -> &KVPrimaryKeyMarker {
        &self.to
    }
}

impl KVScanOp for KVRangeScan {
    fn scan<'a, T: KVScannable + ?Sized>(
        &self,
        scannable: &'a T,
    ) -> Result<Box<dyn KVScanIterator + Send + Sync>, KVRuntimeError> {
        todo!()
    }
}

pub enum KVColumnFilter {
    All,
}

pub struct KVScanExecutor;

impl KVScanExecutor {
    pub fn execute(
        table: &KVTableState,
        scan_op: impl KVScanOp,
    ) -> Result<ScanResultProducer, KVRuntimeError> {
        let pk_schema = KVPrimaryKeySchema::from_columns_schema(&table.columns_schema);
        let mut result_producer = ScanResultProducer::new(pk_schema);

        for scannable in table.list_scannable() {
            // TODO: Scannable check preconditions (bloom filters etc.)

            let scan_result = scan_op.scan(scannable)?;
            result_producer.add_iter(scan_result);
        }

        // let pk_schema = KVPrimaryKeySchema::from_columns_schema(&table.columns_schema);
        // ScanResultItersMerge::new(&pk_schema, result_iters);

        Ok(result_producer)
    }
}

pub struct ScanResultProducer {
    schema: KVPrimaryKeySchema,
    heap: Vec<Box<dyn KVScanIterator + Send + Sync>>,
}

impl ScanResultProducer {
    fn new(schema: KVPrimaryKeySchema) -> Self {
        Self {
            schema,
            heap: vec![],
        }
    }

    fn add_iter(&mut self, iter: Box<dyn KVScanIterator + Send + Sync>) {
        self.heap.push(iter);
        self.heapify();
    }

    fn heapify(&mut self) {
        self.heap.sort_by(|a, b| {
            // TODO: REPLACE THIS
            MutationComparator::cmp(
                &self.schema,
                a.peek_mutation().unwrap().mutation(),
                b.peek_mutation().unwrap().mutation(),
            )
            .unwrap()
        });
        todo!()
    }
}

#[async_trait]
impl KVScanIterator for ScanResultProducer {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        todo!()
    }
    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>> {
        todo!()
    }
}

// pub struct SetScanParams {
//     rows: Box<[Box<[u8]>]>,
//     columns: ColumnFilter,
// }

// impl SetScanParams {
//     pub fn new(rows: Box<[Box<[u8]>]>, columns: ColumnFilter) -> Self {
//         assert!(rows.is_sorted()); //TODO:

//         Self { rows, columns }
//     }
// }
