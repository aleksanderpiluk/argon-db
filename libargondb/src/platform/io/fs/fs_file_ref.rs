use std::{io, path::PathBuf};

use async_trait::async_trait;

use crate::platform::io::{
    BoxFileRef, FileRef, ReadOnlyFileHandle, fs::fs_file_handle::FsReadOnlyFileHandle,
};

#[derive(Clone)]
pub struct FsFileRef {
    path: PathBuf,
}

#[async_trait]
impl FileRef for FsFileRef {
    async fn open_read_only(&self) -> Result<Box<dyn ReadOnlyFileHandle>, io::Error> {
        Ok(Box::new(FsReadOnlyFileHandle::new(&self.path)))
    }

    fn box_clone(&self) -> BoxFileRef {
        Box::new(self.clone())
    }
}
