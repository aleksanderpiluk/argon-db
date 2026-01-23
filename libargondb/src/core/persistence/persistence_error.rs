use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct PersistenceError(pub Box<dyn Error + Send + Sync + 'static>);

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait OrPersistenceError<T> {
    fn ok_or_persistence_error(self) -> Result<T, PersistenceError>;
}

impl<T, E: Error + Send + Sync + 'static> OrPersistenceError<T> for Result<T, E> {
    fn ok_or_persistence_error(self) -> Result<T, PersistenceError> {
        self.map_err(|e| PersistenceError(Box::new(e)))
    }
}
