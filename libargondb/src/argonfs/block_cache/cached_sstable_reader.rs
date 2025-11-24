use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    block_cache::BlockCache,
    kv::{
        KVSSTableBlockPtr, KVSSTableDataBlockIter, KVSSTableReader, KVSSTableStats,
        KVSSTableSummaryIndex, KVScanIteratorItem,
    },
};

pub struct CachedSSTableReader<R: KVSSTableReader + Send + Sync> {
    block_cache: Arc<BlockCache>,
    inner_reader: R,
}

impl<R: KVSSTableReader + Send + Sync> CachedSSTableReader<R> {
    pub fn new(block_cache: Arc<BlockCache>, reader: R) -> Self {
        Self {
            block_cache,
            inner_reader: reader,
        }
    }
}

#[async_trait]
impl<R: KVSSTableReader + Send + Sync> KVSSTableReader for CachedSSTableReader<R> {
    async fn read_stats_and_index(&self) -> (KVSSTableStats, KVSSTableSummaryIndex) {
        // As this function should be called only once to create KVSSTable instance, there's no need to cache it
        self.inner_reader.read_stats_and_index().await
    }

    async fn read_data_block(
        &self,
        ptr: &KVSSTableBlockPtr,
    ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync> {
        let block = self.block_cache.get_block(ptr, true);

        let is_ready: bool = block.is_ready();
        if !is_ready {
            todo!("Wait for block to be ready (read)");
        }

        Box::new(CachedReaderIter {})
    }
}

struct CachedReaderIter {}

impl KVSSTableDataBlockIter for CachedReaderIter {
    fn next(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        todo!()
    }
}
