use std::{io, sync::Arc};

use crate::{
    ArgonFsConfig,
    argonfs::block_cache::BlockCache,
    kv::KVSSTableReader,
    platform::io::{BoxFileSystem, FileSystem},
};

pub struct ArgonFsScanner {
    block_cache: Arc<BlockCache>,
    filesystem: Arc<BoxFileSystem>,
}

impl ArgonFsScanner {
    pub fn new(
        config: &ArgonFsConfig,
        block_cache: Arc<BlockCache>,
        filesystem: Arc<BoxFileSystem>,
    ) -> Self {
        Self {
            block_cache,
            filesystem,
        }
    }

    pub fn scan_table(&self, table_name: &str) -> Result<ArgonFsScanTableResult, io::Error> {
        // let io = self.io_subsystem.platform_io_adapter();
        // let table_dir = self.path_factory.table_dir(table_name);
        let mut sstables = vec![];

        // if !(io.exists(&table_dir).unwrap()) {
        //     return Ok(ArgonFsScanTableResult { sstables });
        // }

        // let dir_content = io.scan_dir(&table_dir).unwrap();

        // for file_path in &dir_content.files {
        //     if file_path.ends_with(".argonfile") {
        //         // let format_reader =
        //         //     ArgonfileFormatReader::new(self.io_subsystem.clone(), file_path);

        //         // sstables.push(CachedSSTableReader::new(
        //         //     self.block_cache.clone(),
        //         //     self.io_subsystem.clone(),
        //         //     format_reader,
        //         // ));
        //     }
        // }

        Ok(ArgonFsScanTableResult { sstables })
    }
}

pub struct ArgonFsScanTableResult {
    pub sstables: Vec<Arc<Box<dyn KVSSTableReader + Send + Sync>>>,
}
