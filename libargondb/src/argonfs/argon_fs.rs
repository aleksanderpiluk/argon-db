use std::sync::Arc;

use thiserror::Error;

use crate::{
    ArgonFsConfig,
    argonfs::{
        argon_fs_worker_pool::ArgonFsWorkerPool,
        argonfile_sstable::{ArgonfileSSTable, ArgonfileSSTableLoadError},
        block_cache::BlockCache,
    },
    kv::{KVScannable, schema::KVTableSchema},
    platform::io::{BoxFileSystem, FileSystemError, fs::FsFileSystem},
};

pub struct ArgonFs {
    block_cache: Arc<BlockCache>,
    filesystem: Arc<BoxFileSystem>,
    worker_pool: Arc<ArgonFsWorkerPool>,
}

impl ArgonFs {
    pub fn init(config: ArgonFsConfig) -> Result<Self, ArgonFsError> {
        let block_cache_config = config.to_block_cache_config();
        let block_cache: Arc<BlockCache> = Arc::new(BlockCache::new(block_cache_config));

        let filesystem: Arc<BoxFileSystem> = Arc::new(Box::new(FsFileSystem::new(
            config.fs_filesystem_config.clone(),
        )));

        let worker_pool = Arc::new(ArgonFsWorkerPool::new(1));

        Ok(Self {
            block_cache,
            filesystem,
            worker_pool,
        })
    }

    pub fn scan_for_sstables(
        &self,
        table_schema: &KVTableSchema,
    ) -> Result<Vec<Box<dyn KVScannable>>, ArgonFsError> {
        todo!()
    }

    pub async fn scan_sstables(
        &self,
        table_name: &str,
    ) -> Result<Vec<Box<dyn KVScannable>>, ArgonFsError> {
        let sstable_refs = self.filesystem.scan_table_catalog(table_name).await?;

        let mut sstables: Vec<Box<dyn KVScannable>> = vec![];
        for file_ref in sstable_refs {
            let argonfile_sstable = ArgonfileSSTable::load(
                self.block_cache.clone(),
                self.worker_pool.clone(),
                file_ref,
            )
            .await?;
            sstables.push(Box::new(argonfile_sstable));
        }

        Ok(sstables)
    }
}

#[derive(Error, Debug)]
pub enum ArgonFsError {
    #[error("argonfile load error - {0}")]
    ArgonfileLoadError(#[from] ArgonfileSSTableLoadError),
    #[error("file system error - {0}")]
    FileSystemError(#[from] FileSystemError),
}
