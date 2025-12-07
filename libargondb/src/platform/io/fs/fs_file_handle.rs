use std::io::SeekFrom;

use async_trait::async_trait;

use crate::platform::io::{FileHandleError, ReadData, ReadOnlyFileHandle};

pub struct FsReadOnlyFileHandle;

#[async_trait]
impl ReadOnlyFileHandle for FsReadOnlyFileHandle {
    async fn read(&self, buf_size: usize) -> Result<ReadData, FileHandleError> {
        todo!()
    }

    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileHandleError> {
        todo!()
    }
}
