use std::fmt::Display;

use crate::exit::abort_on_critical_error;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct CriticalError {
    msg: Option<String>,
    source: Option<BoxError>,
}

impl CriticalError {
    pub fn from_msg(msg: impl AsRef<str>) -> Self {
        Self {
            msg: Some(msg.as_ref().to_string()),
            source: None,
        }
    }

    pub fn from_msg_and_source(
        msg: impl AsRef<str>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            msg: Some(msg.as_ref().to_string()),
            source: Some(Box::new(source)),
        }
    }

    pub fn from_source(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            msg: None,
            source: Some(Box::new(source)),
        }
    }
}

impl std::error::Error for CriticalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl Display for CriticalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.msg, &self.source) {
            (Some(msg), Some(source)) => {
                write!(f, "CriticalError with msg: {}, source: {}", msg, source)
            }
            (Some(msg), None) => write!(f, "CriticalError with msg: {}", msg),
            (None, Some(source)) => write!(f, "CriticalError with source: {}", source),
            _ => write!(f, "CriticalError"),
        }
    }
}

pub type CriticalResult<T> = Result<T, CriticalError>;

pub trait OrCriticalError<T> {
    fn ok_or_critical_err(self) -> CriticalResult<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> OrCriticalError<T> for Result<T, E> {
    fn ok_or_critical_err(self) -> CriticalResult<T> {
        self.map_err(|e| CriticalError::from_source(e))
    }
}

// impl<T> OrCriticalError<T> for Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
//     fn ok_or_critical_err(self) -> CriticalResult<T> {
//         self.map_err(|e| CriticalError::from_source(e))
//     }
// }

pub trait OkOrAbort<T> {
    fn ok_or_abort(self) -> T;
}

impl<T> OkOrAbort<T> for CriticalResult<T> {
    fn ok_or_abort(self) -> T {
        match self {
            Self::Ok(val) => val,
            Self::Err(err) => abort_on_critical_error(err),
        }
    }
}
