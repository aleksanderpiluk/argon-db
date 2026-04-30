use async_trait::async_trait;

use crate::kv::{
    KVTableSchema,
    error::KVRuntimeError,
    mutation::KVMutation,
    primary_key::{KVPrimaryKeyMarker, KVPrimaryKeyMarkerUtils, KVPrimaryKeyUtils},
};

#[async_trait]
pub trait KVScanOp: std::fmt::Display {
    async fn scan<T: KVScannable + Send + Sync + ?Sized>(
        &self,
        scannable: &T,
    ) -> Result<KVRangeScanResult, KVRuntimeError>;
}

#[async_trait]
pub trait KVScannable: Send + Sync + std::fmt::Display {
    async fn range_scan(&self, scan: &KVRangeScan) -> Result<KVRangeScanResult, KVRuntimeError>;
    async fn row_scan(&self, primary_key: &[u8]) -> Result<KVRangeScanResult, KVRuntimeError>;
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
    schema: KVTableSchema,
    from: KVPrimaryKeyMarker,
    to: KVPrimaryKeyMarker,
    columns: KVColumnFilter,
}

impl KVRangeScan {
    pub fn new(
        schema: KVTableSchema,
        from: KVPrimaryKeyMarker,
        to: KVPrimaryKeyMarker,
        columns: KVColumnFilter,
    ) -> Self {
        Self {
            schema,
            from,
            to,
            columns,
        }
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

impl std::fmt::Display for KVRangeScan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "KVRangeScan(from={}, to={})",
            KVPrimaryKeyMarkerUtils::debug_fmt(&self.schema, &self.from).unwrap(),
            KVPrimaryKeyMarkerUtils::debug_fmt(&self.schema, &self.to).unwrap()
        )
    }
}

pub struct KVRowScan {
    schema: KVTableSchema,
    primary_key: Box<[u8]>,
    columns: KVColumnFilter,
}

impl KVRowScan {
    pub fn new(schema: KVTableSchema, primary_key: Box<[u8]>, columns: KVColumnFilter) -> Self {
        Self {
            schema,
            primary_key,
            columns,
        }
    }
}

#[async_trait]
impl KVScanOp for KVRowScan {
    async fn scan<T: KVScannable + Send + Sync + ?Sized>(
        &self,
        scannable: &T,
    ) -> Result<KVRangeScanResult, KVRuntimeError> {
        scannable.row_scan(&self.primary_key).await
    }
}

impl std::fmt::Display for KVRowScan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "KVRowScan(primary_key={})",
            KVPrimaryKeyUtils::debug_fmt(&self.schema, &self.primary_key).unwrap()
        )
    }
}

pub enum KVColumnFilter {
    All,
}
