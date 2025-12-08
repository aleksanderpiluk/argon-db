use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FsFileSystemConfig {
    pub fs_read_pool_thread_count: usize,
    pub storage_root: PathBuf,
}

impl Default for FsFileSystemConfig {
    fn default() -> Self {
        Self {
            fs_read_pool_thread_count: 1,
            storage_root: "/etc/argondb/storage".into(),
        }
    }
}
