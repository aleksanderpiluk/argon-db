use std::{path::Path, sync::Arc};

use crate::{
    ArgonFsConfig,
    argonfs::{
        argonfile_sstable::ArgonfileSSTable,
        argonfs_scanner::{ArgonFsScanTableResult, ArgonFsScanner},
        block_cache::BlockCache,
    },
    kv::KVScannable,
    platform::io::{BoxFileSystem, FileSystem, fs::FsFileSystem},
};

pub struct ArgonFs {
    block_cache: Arc<BlockCache>,
    filesystem: Arc<BoxFileSystem>,
    scanner: Arc<ArgonFsScanner>,
}

impl ArgonFs {
    pub fn init(config: ArgonFsConfig) -> Result<Self, ArgonFsInitError> {
        let block_cache_config = config.to_block_cache_config();
        let block_cache: Arc<BlockCache> = Arc::new(BlockCache::new(block_cache_config));

        let filesystem: Arc<BoxFileSystem> = Arc::new(Box::new(FsFileSystem::new(
            config.fs_filesystem_config.clone(),
        )));

        let scanner = Arc::new(ArgonFsScanner::new(
            &config,
            block_cache.clone(),
            filesystem.clone(),
        ));

        Ok(Self {
            block_cache,
            filesystem,
            scanner,
        })
    }

    pub fn scan_table(&self, table_name: &str) -> ArgonFsScanTableResult {
        self.scanner.scan_table(table_name).unwrap()
    }

    pub async fn open_sstable(&self, p: impl AsRef<Path>) -> Box<dyn KVScannable> {
        // Box::new(CachedSSTableReader::new(
        //     self.block_cache.clone(),
        //     self.io_subsystem.clone(),
        //     Arc::new(Box::new(ArgonfileFormatReader::new(
        //         self.io_subsystem.clone(),
        //         p,
        //     ))),
        // ))
        let file_ref = todo!();
        let sstable = ArgonfileSSTable::load(self.block_cache.clone(), file_ref)
            .await
            .unwrap();
        Box::new(sstable)
    }
}

#[derive(Debug)]
pub enum ArgonFsInitError {
    IOSubsystemInitError,
}
