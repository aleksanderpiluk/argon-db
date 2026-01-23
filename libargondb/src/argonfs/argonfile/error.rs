use std::io;

#[derive(Debug)]
pub struct ArgonfileBuilderError {
    msg: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl ArgonfileBuilderError {
    pub fn from_msg(msg: impl AsRef<str>) -> Self {
        Self {
            msg: Some(msg.as_ref().to_string()),
            source: None,
        }
    }

    pub fn from_source(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            msg: None,
            source: Some(Box::new(source)),
        }
    }
}

impl std::error::Error for ArgonfileBuilderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl std::fmt::Display for ArgonfileBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.msg, &self.source) {
            (Some(msg), Some(source)) => {
                write!(f, "ArgonfileBuilderError msg: {} source: {}", msg, source)
            }
            (Some(msg), None) => write!(f, "ArgonfileBuilderError msg: {}", msg),
            (None, Some(source)) => write!(f, "ArgonfileBuilderError source: {}", source),
            _ => write!(f, "ArgonfileBuilderError"),
        }
    }
}

impl From<ArgonfileWriterError> for ArgonfileBuilderError {
    fn from(value: ArgonfileWriterError) -> Self {
        Self {
            msg: Some("ArgonfileWriterError".into()),
            source: None,
        }
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

// #[derive(Error, Debug)]
// pub enum ArgonfileDeserializeError {
//     #[error("invalid buffer size")]
//     InvalidBufferSize,
//     #[error("checksum parse error")]
//     ChecksumTypeParseError(#[from] ChecksumTypeParseError),
//     #[error("compression type parse")]
//     CompressionTypeParseError(#[from] CompressionTypeParseError),
//     #[error("conversion to type error")]
//     TryFromSliceError(#[from] TryFromSliceError),
// }

#[derive(Debug)]
pub struct ArgonfileParseError;

pub type ArgonfileParseResult<T> = Result<T, ArgonfileParseError>;
