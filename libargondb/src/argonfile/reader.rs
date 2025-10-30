use crate::kv::{KVSSTableDataBlockIter, KVSSTableReader, KVSSTableStats, KVSSTableSummaryIndex};
use async_trait::async_trait;
use std::path::Path;

pub struct ArgonfileReader {}

impl ArgonfileReader {
    pub fn new(p: impl AsRef<Path>) -> Self {
        todo!()
    }
}

#[async_trait]
impl KVSSTableReader for ArgonfileReader {
    async fn read_stats_and_index(&self) -> (KVSSTableStats, KVSSTableSummaryIndex) {
        todo!()
    }

    async fn read_data_block(&self, ptr: &()) -> Box<dyn KVSSTableDataBlockIter + Send + Sync> {
        let tag = todo!();
        // let block = self.block_cache.get_block(tag, true);

        todo!()
    }
}
