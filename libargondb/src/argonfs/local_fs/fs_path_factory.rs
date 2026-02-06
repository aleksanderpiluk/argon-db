use std::path::{Path, PathBuf};

use crate::{
    argonfs::local_fs::FsFileSystemConfig,
    kv::{Id, ObjectId},
};

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

    pub fn table_dir(&self, table_id: &Id) -> PathBuf {
        self.tables_root.join(table_id.as_ref())
    }

    pub fn sstable_file(&self, table_id: &Id, sstable_id: ObjectId) -> PathBuf {
        self.table_dir(table_id)
            .join(format!("{}.argonfile", sstable_id.0))
    }

    pub fn state_snapshot_file(&self) -> PathBuf {
        self.config.storage_root.join("_state_snapshot")
    }
}
