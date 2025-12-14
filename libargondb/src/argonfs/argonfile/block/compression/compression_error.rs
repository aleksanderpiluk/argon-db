use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("write error - {0}")]
    WriteError(io::Error),
}
