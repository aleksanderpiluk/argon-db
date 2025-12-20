use std::vec;

use async_trait::async_trait;

use crate::kv::{
    error::KVRuntimeError,
    mutation::{KVMutation, MutationComparator},
    primary_key::{KVPrimaryKeyMarker, KVPrimaryKeySchema},
    scan_iter::KVMergeScanIter,
    table::KVTableState,
};

#[async_trait]
pub trait KVScanOp {
    async fn scan<T: KVScannable + Send + Sync + ?Sized>(
        &self,
        scannable: &T,
    ) -> Result<KVRangeScanResult, KVRuntimeError>;
}

#[async_trait]
pub trait KVScannable: Send + Sync {
    async fn range_scan(&self, scan: &KVRangeScan) -> Result<KVRangeScanResult, KVRuntimeError>;
    // fn set_scan(&self, scan: SetScanParams) -> impl ScanResultIter;
}

pub enum KVRangeScanResult {
    Empty,
    Iter(Box<dyn KVScanIterator + Send + Sync>),
}

#[async_trait]
pub trait KVScanIterator {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>>;
    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>>;
}

pub trait KVScanIteratorItem {
    fn primary_key(&self) -> &[u8];

    fn mutation(&self) -> &(dyn KVMutation + Send + Sync);
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

#[async_trait]
impl KVScanOp for KVRangeScan {
    async fn scan<T: KVScannable + Send + Sync + ?Sized>(
        &self,
        scannable: &T,
    ) -> Result<KVRangeScanResult, KVRuntimeError> {
        scannable.range_scan(&self).await
    }
}

pub enum KVColumnFilter {
    All,
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
