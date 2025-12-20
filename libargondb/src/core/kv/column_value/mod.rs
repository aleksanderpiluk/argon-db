use crate::kv::{
    KVRuntimeError,
    column_type::{ColumnTypeSerialize, ColumnTypeText, ColumnTypeU16, ColumnTypeU16Array},
};

pub trait KVColumnValue {
    fn serialize(&self) -> Result<Box<[u8]>, KVRuntimeError>;
}

pub struct KVColumnValueBuilder;

impl KVColumnValueBuilder {
    pub fn text(value: String) -> Box<dyn KVColumnValue> {
        Box::new(ColumnValueText(value))
    }

    pub fn u16(value: u16) -> Box<dyn KVColumnValue> {
        Box::new(ColumnValueU16(value))
    }

    pub fn u16_array(value: Vec<u16>) -> Box<dyn KVColumnValue> {
        Box::new(ColumnValueU16Array(value))
    }
}

struct ColumnValueText(String);

impl KVColumnValue for ColumnValueText {
    fn serialize(&self) -> Result<Box<[u8]>, KVRuntimeError> {
        ColumnTypeText::serialize(&self.0)
    }
}

struct ColumnValueU16(u16);

impl KVColumnValue for ColumnValueU16 {
    fn serialize(&self) -> Result<Box<[u8]>, KVRuntimeError> {
        ColumnTypeU16::serialize(self.0)
    }
}

struct ColumnValueU16Array(Vec<u16>);

impl KVColumnValue for ColumnValueU16Array {
    fn serialize(&self) -> Result<Box<[u8]>, KVRuntimeError> {
        ColumnTypeU16Array::serialize(&self.0)
    }
}
