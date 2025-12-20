use std::io::{self, SeekFrom, Write};

use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait ReadOnlyFileHandle: Send + Sync {
    async fn read(&mut self, buf_size: usize) -> Result<ReadData, FileHandleError>;

    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileHandleError>;

    async fn seek_and_read(
        &mut self,
        pos: SeekFrom,
        buf_size: usize,
    ) -> Result<ReadData, FileHandleError> {
        self.seek(pos).await?;
        self.read(buf_size).await
    }
}

#[derive(Error, Debug)]
#[error("file handle error {0}")]
pub enum FileHandleError {
    IOError(#[from] io::Error),
    SeekError(String),
}

pub struct ReadData(Box<dyn AsRef<[u8]> + Send + Sync>);

impl From<Box<dyn AsRef<[u8]> + Send + Sync>> for ReadData {
    fn from(value: Box<dyn AsRef<[u8]> + Send + Sync>) -> Self {
        Self(value)
    }
}

impl AsRef<[u8]> for ReadData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

#[async_trait]
pub trait WriteOnlyFileHandle: Write + Send + Sync {}
