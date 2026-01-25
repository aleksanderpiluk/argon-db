use async_trait::async_trait;

use crate::kv::{ObjectId, error::KVRuntimeError, mutation::KVMutation, scan::KVScanIteratorItem};

use super::scan::KVScannable;

pub trait KVSSTable: KVScannable {
    fn level(&self) -> u64;
    fn sstable_id(&self) -> ObjectId;
    fn mutation_count(&self) -> u64;
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
