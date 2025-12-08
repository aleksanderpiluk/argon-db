use std::io;

use thiserror::Error;

#[derive(Debug)]
pub enum ArgonfileBuilderError {
    WriterError(ArgonfileWriterError),
    AssertionError,
}

impl From<ArgonfileWriterError> for ArgonfileBuilderError {
    fn from(value: ArgonfileWriterError) -> Self {
        Self::WriterError(value)
    }
}

#[derive(Debug)]
pub enum ArgonfileWriterError {
    IOError(io::Error),
    PartialWrite(usize),
}

impl From<io::Error> for ArgonfileWriterError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

#[derive(Error, Debug)]
pub enum ArgonfileDeserializeError {
    #[error("invalid buffer size")]
    InvalidBufferSize,
}
