use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    block_cache::BlockCache,
    kv::{KVSSTableDataBlockIter, KVSSTableReader},
};

struct ArgonfileReader {
    block_cache: Arc<BlockCache>,
}

#[async_trait]
impl KVSSTableReader for ArgonfileReader {
    async fn initial_read(&self) {
        todo!()
    }

    async fn read_data_block(&self, ptr: &()) -> Box<dyn KVSSTableDataBlockIter + Send + Sync> {
        let tag = todo!();
        let block = self.block_cache.get_block(tag, true);

        todo!()
    }
}
