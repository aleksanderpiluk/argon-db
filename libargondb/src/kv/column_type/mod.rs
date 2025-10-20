mod bytes;

pub use bytes::Bytes;

use std::cmp::Ordering;

pub trait ColumnType {
    fn eq(this: &[u8], that: &[u8]) -> bool;
    fn cmp(this: &[u8], that: &[u8]) -> Ordering;
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ColumnTypeCode {
    Bytes = 1,
}

impl ColumnTypeCode {
    pub fn type_for_code(code: u8) -> Option<impl ColumnType> {
        if code == 1 { Some(Bytes) } else { todo!() }
    }
}
