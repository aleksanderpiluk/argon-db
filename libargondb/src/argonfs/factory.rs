use std::{path::Path, sync::Arc};

use crate::{
    argonfs::{
        argonfile::ArgonfileFormatReader,
        block_cache::{BlockCache, BlockCacheConfig},
        cached_sstable_reader::CachedSSTableReader,
        io_subsystem::BoxIOSubsystem,
    },
    kv::KVSSTableReader,
};

pub struct ArgonFsFactory {
    block_cache: Arc<BlockCache>,
    io_subsystem: Arc<BoxIOSubsystem>,
}

// DefaultIOSubsystem::new()
impl ArgonFsFactory {
    pub fn new(io_subsystem: Arc<BoxIOSubsystem>) -> Self {
        let block_cache: Arc<BlockCache> = Arc::new(BlockCache::new(BlockCacheConfig {
            page_size: 1 << 13,
            pages_total: 1 << 23,
        }));

        Self {
            block_cache,
            io_subsystem,
        }
    }

    pub fn open_sstable(&self, p: impl AsRef<Path>) -> Arc<Box<dyn KVSSTableReader + Send + Sync>> {
        Arc::new(Box::new(CachedSSTableReader::new(
            self.block_cache.clone(),
            self.io_subsystem.clone(),
            Arc::new(Box::new(ArgonfileFormatReader::new(p))),
        )))
    }
}
