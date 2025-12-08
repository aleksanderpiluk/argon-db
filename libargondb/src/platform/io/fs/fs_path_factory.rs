use std::path::{Path, PathBuf};

use crate::platform::io::fs::FsFileSystemConfig;

pub struct FsPathFactory {
    config: FsFileSystemConfig,
    tables_root: PathBuf,
}

impl FsPathFactory {
    pub fn new(config: FsFileSystemConfig) -> Self {
        let tables_root = config.storage_root.join("tables");

        Self {
            config,
            tables_root,
        }
    }

    pub fn tables_root(&self) -> &Path {
        &self.tables_root
    }

    pub fn table_dir(&self, table_name: &str) -> PathBuf {
        self.tables_root.join(table_name)
    }
}
