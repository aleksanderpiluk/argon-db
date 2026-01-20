use std::{
    fs, io,
    path::{Path, PathBuf},
};

use async_trait::async_trait;

use crate::argonfs::{
    fs::{BoxFileRef, FileRef, ReadOnlyFileHandle, WriteOnlyFileHandle},
    local_fs::fs_file_handle::{FsReadOnlyFileHandle, FsWriteOnlyFileHandle},
};

#[derive(Clone)]
pub struct FsFileRef {
    path: PathBuf,
}

impl FsFileRef {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
        }
    }
}

#[async_trait]
impl FileRef for FsFileRef {
    async fn open_read_only(&self) -> Result<Box<dyn ReadOnlyFileHandle>, io::Error> {
        Ok(Box::new(FsReadOnlyFileHandle::new(&self.path)?))
    }

    async fn open_write_only(&self) -> Result<Box<dyn WriteOnlyFileHandle>, io::Error> {
        Ok(Box::new(FsWriteOnlyFileHandle::new(&self.path)?))
    }

    async fn remove(self: Box<Self>) -> Result<(), io::Error> {
        fs::remove_file(self.path)
    }

    fn box_clone(&self) -> BoxFileRef {
        Box::new(self.clone())
    }
}
