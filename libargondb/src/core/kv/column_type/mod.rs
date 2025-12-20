mod bytes;
mod text;
mod u16;
mod u16_array;

pub use bytes::ColumnTypeBytes;
pub use text::ColumnTypeText;
pub use u16::ColumnTypeU16;
pub use u16_array::ColumnTypeU16Array;

use std::cmp::Ordering;

use crate::kv::{KVRuntimeError, KVRuntimeErrorKind};

pub trait ColumnType {
    fn eq(&self, this: &[u8], that: &[u8]) -> bool;
    fn cmp(&self, this: &[u8], that: &[u8]) -> Ordering;
}

pub trait ColumnTypeDeserialize {
    type Output;

    fn deserialize(buf: &[u8]) -> Result<Self::Output, KVRuntimeError>;
}

pub trait ColumnTypeSerialize {
    type Input<'a>
    where
        Self: 'a;

    fn serialize(input: Self::Input<'_>) -> Result<Box<[u8]>, KVRuntimeError>;
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ColumnTypeCode {
    Bytes = 1,
    Text = 2,
    U16 = 3,
    U16Array = 4,
}

impl TryFrom<u8> for ColumnTypeCode {
    type Error = KVRuntimeError;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            1 => Ok(ColumnTypeCode::Bytes),
            2 => Ok(ColumnTypeCode::Text),
            3 => Ok(ColumnTypeCode::U16),
            4 => Ok(ColumnTypeCode::U16Array),
            _ => Err(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::DataMalformed,
                format!(
                    "failed to parse type code - unknown column type code: {}",
                    code
                ),
            )),
        }
    }
}

impl ColumnTypeCode {
    pub fn type_for_code(code: u8) -> Result<Box<dyn ColumnType>, KVRuntimeError> {
        match code {
            1 => Ok(Box::new(ColumnTypeBytes)),
            2 => Ok(Box::new(ColumnTypeText)),
            3 => Ok(Box::new(ColumnTypeU16)),
            4 => Ok(Box::new(ColumnTypeU16Array)),
            _ => Err(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::DataMalformed,
                format!(
                    "failed to get type for code - unknown column type code: {}",
                    code
                ),
            )),
        }
    }
}
