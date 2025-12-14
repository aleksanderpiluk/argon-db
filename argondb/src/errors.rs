use libargondb::{ArgonFsError, kv::schema::KVTableSchemaBuildError};

pub struct CriticalError;

pub type CriticalResult<T> = Result<T, CriticalError>;

pub trait OrCriticalError<T> {
    fn ok_or_critical_err(self) -> CriticalResult<T>;
}

impl<T> OrCriticalError<T> for Result<T, KVTableSchemaBuildError> {
    fn ok_or_critical_err(self) -> CriticalResult<T> {
        self.map_err(|e| CriticalError)
    }
}

impl<T> OrCriticalError<T> for Result<T, ArgonFsError> {
    fn ok_or_critical_err(self) -> CriticalResult<T> {
        self.map_err(|e| CriticalError)
    }
}
