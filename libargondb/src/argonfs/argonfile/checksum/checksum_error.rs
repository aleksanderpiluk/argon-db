use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChecksumError {
    #[error("checksum is malformed")]
    ChecksumMalformed,
    #[error("checksum validation failed")]
    ValidationFailed,
    #[error("write error - {0}")]
    WriteError(io::Error),
}
