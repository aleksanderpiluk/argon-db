use std::path::PathBuf;

use crate::argonfs::{block_cache::BlockCacheConfig, local_fs::FsFileSystemConfig};

#[derive(Debug, Clone)]
pub struct ArgonFsConfig {
    pub fs_filesystem_config: FsFileSystemConfig,
    pub block_cache_page_size: usize,
    pub block_cache_pages_count: usize,
}

impl Default for ArgonFsConfig {
    fn default() -> Self {
        Self {
            fs_filesystem_config: FsFileSystemConfig::default(),

            block_cache_page_size: 1 << 13,   // page size = 8KB
            block_cache_pages_count: 1 << 15, // total pages size = 256MB
        }
    }
}

impl ArgonFsConfig {
    pub fn to_block_cache_config(&self) -> BlockCacheConfig {
        BlockCacheConfig {
            page_size: self.block_cache_page_size,
            pages_total: self.block_cache_pages_count,
        }
    }
}
