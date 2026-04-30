use std::io;

use thiserror::Error;

use crate::argonfs::argonfile::error::ArgonfileParseError;

#[derive(Error, Debug)]
pub enum ChecksumError {
    #[error("checksum is malformed")]
    ChecksumMalformed,
    #[error("checksum validation failed")]
    ValidationFailed,
    #[error("write error - {0}")]
    WriteError(io::Error),
}

impl From<ChecksumError> for ArgonfileParseError {
    fn from(value: ChecksumError) -> Self {
        Self
    }
}
