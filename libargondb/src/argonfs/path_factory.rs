use std::path::{Path, PathBuf};

use crate::argonfs::config::ArgonFsConfig;

pub struct ArgonFsPathFactory {
    config: ArgonFsConfig,
    tables_root: PathBuf,
}

impl ArgonFsPathFactory {
    pub fn new(config: ArgonFsConfig) -> Self {
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
