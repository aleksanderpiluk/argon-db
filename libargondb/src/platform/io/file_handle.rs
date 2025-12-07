use std::io::SeekFrom;

use async_trait::async_trait;

#[async_trait]
pub trait ReadOnlyFileHandle {
    async fn read(&self, buf_size: usize) -> Result<ReadData, FileHandleError>;

    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileHandleError>;
}

pub struct FileHandleError;

pub struct ReadData;
