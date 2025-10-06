mod bytes;

pub use bytes::Bytes;

use std::cmp::Ordering;

pub trait ColumnType {
    fn eq(this: &[u8], that: &[u8]) -> bool;
    fn cmp(this: &[u8], that: &[u8]) -> Ordering;
}
