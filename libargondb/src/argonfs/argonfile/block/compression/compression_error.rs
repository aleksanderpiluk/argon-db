use std::io;

use thiserror::Error;

use crate::argonfs::argonfile::error::ArgonfileParseError;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("write error - {0}")]
    WriteError(io::Error),
}

impl From<CompressionError> for ArgonfileParseError {
    fn from(value: CompressionError) -> Self {
        todo!()
    }
}
