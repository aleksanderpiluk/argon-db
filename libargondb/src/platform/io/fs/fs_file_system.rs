use std::fs;

use async_trait::async_trait;
use crossbeam::queue::ArrayQueue;

use crate::platform::io::{
    BoxFileRef, FileSystem, FileSystemError, TableCatalogRef,
    fs::{FsFileSystemConfig, fs_file_ref::FsFileRef, fs_path_factory::FsPathFactory},
};

pub struct FsFileSystem {
    ctx: FsFileSystemCtx,
}

impl FsFileSystem {
    pub fn new(config: FsFileSystemConfig) -> Self {
        Self {
            ctx: FsFileSystemCtx::new(config),
        }
    }
}

#[async_trait]
impl FileSystem for FsFileSystem {
    async fn scan_table_catalogs(&self) -> Result<Vec<Box<dyn TableCatalogRef>>, FileSystemError> {
        let tables_root_dir = self.ctx.path_factory.tables_root();
        let dir_entries = fs::read_dir(tables_root_dir)?;

        let refs = vec![];

        for entry in dir_entries {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                todo!();
            }
        }

        Ok(refs)
    }

    async fn scan_table_catalog(
        &self,
        table_name: &str,
    ) -> Result<Vec<BoxFileRef>, FileSystemError> {
        let table_dir = self.ctx.path_factory.table_dir(table_name);
        let mut refs: Vec<BoxFileRef> = vec![];

        if table_dir.exists() {
            let dir_entries = fs::read_dir(table_dir)?;

            for entry in dir_entries {
                let entry = entry?;

                if let Some(extension) = entry.path().extension()
                    && extension == "argonfile"
                {
                    refs.push(Box::new(FsFileRef::new(&entry.path())));
                }
            }
        }

        Ok(refs)
    }
}

struct FsFileSystemCtx {
    queue: ArrayQueue<FsReadRequest>,
    path_factory: FsPathFactory,
}

impl FsFileSystemCtx {
    fn new(config: FsFileSystemConfig) -> Self {
        Self {
            queue: ArrayQueue::new(64),
            path_factory: FsPathFactory::new(config),
        }
    }
}

pub struct FsReadRequest {
    // pub block_tag: BlockTag,
    // pub sstable_format_reader: Arc<BoxedArgonFsFormatReader>,
    // pub sstable_ptr: KVSSTableBlockPtr,
}

struct FsReadWorker {
    // block_cache: Arc<BlockCache>,
    // fs_read_request_queue: Arc<FsReadRequestQueue>,
}

impl FsReadWorker {
    fn run(self) {}

    fn execute_read_request(&self, read_request: FsReadRequest) {
        // let block_cache = self.block_cache.clone();
        // let format_reader = read_request.sstable_format_reader;
        // let block_tag = read_request.block_tag;
        // let sstable_ptr = read_request.sstable_ptr;

        // let mut block_alloc = BlockCacheAllocator::new(block_cache.clone(), block_tag);
        // let block_size = format_reader.load_data_block(sstable_ptr, &mut block_alloc);

        // // let mut block = block_alloc.into_block();
        // let mut block = block_cache.get_block(&block_tag, false).to_exclusive();
        // let wakers = block.set_state_loaded_block(block_size);
        // for waker in wakers {
        //     waker.wake();
        // }
    }
}
