use std::io::{self, SeekFrom};

use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait ReadOnlyFileHandle {
    async fn read(&self, buf_size: usize) -> Result<ReadData, FileHandleError>;

    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileHandleError>;
}

#[derive(Error, Debug)]
#[error("file handle error")]
pub enum FileHandleError {
    IOError(#[from] io::Error),
}

pub struct ReadData(Box<dyn AsRef<[u8]>>);

impl From<Box<dyn AsRef<[u8]>>> for ReadData {
    fn from(value: Box<dyn AsRef<[u8]>>) -> Self {
        Self(value)
    }
}

impl AsRef<[u8]> for ReadData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}
