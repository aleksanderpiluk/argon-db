use std::fmt::Debug;

use async_trait::async_trait;

use crate::kv::{ObjectId, error::KVRuntimeError, mutation::KVMutation, scan::KVScanIteratorItem};

use super::scan::KVScannable;

pub trait KVSSTable: KVScannable {
    fn level(&self) -> u64;
    fn sstable_id(&self) -> ObjectId;
    fn mutation_count(&self) -> u64;
}

#[async_trait]
pub trait KVSSTableReader {
    async fn read_data_block(
        &self,
        ptr: &KVSSTableBlockPtr,
    ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync>;
}

pub trait KVSSTableDataBlockIter {
    fn next(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>>;
}

#[async_trait]
pub trait KVSSTableBuilder {
    /**
     * Add next mutation to builded SSTable file. In caller responsibility is to ensure that next mutations are passed in strict order. If given mutation breaks ordering, implementation should error.
     */
    async fn add_mutation(
        &mut self,
        mutation: &(dyn KVMutation + Send + Sync),
    ) -> Result<(), KVRuntimeError>;
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct KVSSTableBlockPtr(u64, u32);

impl KVSSTableBlockPtr {
    pub fn new(offset: u64, on_disk_size: u32) -> Self {
        Self(offset, on_disk_size)
    }

    pub fn offset(&self) -> u64 {
        self.0
    }

    pub fn on_disk_size(&self) -> u32 {
        self.1
    }
}

impl Debug for KVSSTableBlockPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KVSSTableBlockPtr")
            .field("offset", &self.offset())
            .field("on_disk_size", &self.on_disk_size())
            .finish()
    }
}
