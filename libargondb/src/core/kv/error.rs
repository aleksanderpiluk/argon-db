use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub enum KVConstructorError {
    InvalidData,
}

impl std::error::Error for KVConstructorError {}

impl std::fmt::Display for KVConstructorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidData => write!(f, "KVConstructorError - Invalid data"),
        }
    }
}

#[derive(Debug)]
pub struct KVRuntimeError {
    kind: KVRuntimeErrorKind,
    msg: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl KVRuntimeError {
    pub fn with_msg_and_source(
        kind: KVRuntimeErrorKind,
        msg: impl AsRef<str>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            msg: Some(msg.as_ref().to_string()),
            source: Some(Box::new(source)),
        }
    }

    pub fn with_msg(kind: KVRuntimeErrorKind, msg: impl AsRef<str>) -> Self {
        Self {
            kind,
            msg: Some(msg.as_ref().to_string()),
            source: None,
        }
    }

    pub fn with_source(
        kind: KVRuntimeErrorKind,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            msg: None,
            source: Some(Box::new(source)),
        }
    }
}

impl std::error::Error for KVRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl std::fmt::Display for KVRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = self.kind;
        match (&self.msg, &self.source) {
            (Some(msg), Some(source)) => write!(
                f,
                "KVRuntimeError kind: {}, msg: {}, source: {}",
                kind, msg, source
            ),
            (Some(msg), None) => write!(f, "KVRuntimeError kind: {}, msg: {}", kind, msg),
            (None, Some(source)) => write!(f, "KVRuntimeError kind: {}, source: {}", kind, source),
            _ => write!(f, "KVRuntimeError kind: {}", kind),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KVRuntimeErrorKind {
    IndexOutOfBounds,
    DataMalformed,
    OperationNotAllowed,
}

impl std::fmt::Display for KVRuntimeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IndexOutOfBounds => write!(f, "IndexOutOfBounds"),
            Self::DataMalformed => write!(f, "DataMalformed"),
            Self::OperationNotAllowed => write!(f, "OperationNotAllowed"),
        }
    }
}
