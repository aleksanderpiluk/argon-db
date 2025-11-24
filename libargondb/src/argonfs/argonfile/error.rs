use std::io;

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
