use std::{path::Path, sync::Arc};

use crate::{
    ArgonFsConfig,
    argonfs::{
        argonfile_sstable::ArgonfileSSTable,
        argonfs_scanner::{ArgonFsScanTableResult, ArgonFsScanner},
        block_cache::BlockCache,
        io_subsystem::{IOSubsystem, IOSubsystemInitError},
    },
    kv::KVScannable,
};

pub struct ArgonFs {
    block_cache: Arc<BlockCache>,
    io_subsystem: Arc<IOSubsystem>,
    scanner: Arc<ArgonFsScanner>,
}

impl ArgonFs {
    pub fn init(config: ArgonFsConfig) -> Result<Self, ArgonFsInitError> {
        let block_cache_config = config.to_block_cache_config();
        let block_cache: Arc<BlockCache> = Arc::new(BlockCache::new(block_cache_config));

        let io_subsystem = Arc::new(IOSubsystem::init()?);

        let scanner = Arc::new(ArgonFsScanner::new(
            &config,
            block_cache.clone(),
            io_subsystem.clone(),
        ));

        Ok(Self {
            block_cache,
            io_subsystem,
            scanner,
        })
    }

    pub fn scan_table(&self, table_name: &str) -> ArgonFsScanTableResult {
        self.scanner.scan_table(table_name).unwrap()
    }

    pub fn open_sstable(&self, p: impl AsRef<Path>) -> Box<dyn KVScannable> {
        // Box::new(CachedSSTableReader::new(
        //     self.block_cache.clone(),
        //     self.io_subsystem.clone(),
        //     Arc::new(Box::new(ArgonfileFormatReader::new(
        //         self.io_subsystem.clone(),
        //         p,
        //     ))),
        // ))
        Box::new(ArgonfileSSTable::new(self.block_cache.clone()))
    }
}

#[derive(Debug)]
pub enum ArgonFsInitError {
    IOSubsystemInitError,
}

impl From<IOSubsystemInitError> for ArgonFsInitError {
    fn from(value: IOSubsystemInitError) -> Self {
        Self::IOSubsystemInitError
    }
}
