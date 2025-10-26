use std::io;

pub enum ArgonfileWriterError {
    IOError(io::Error),
    PartialWrite(usize),
}

impl From<io::Error> for ArgonfileWriterError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}
