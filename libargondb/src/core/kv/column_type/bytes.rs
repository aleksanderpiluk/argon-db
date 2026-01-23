use crate::kv::{
    KVRuntimeError,
    column_type::{ColumnType, ColumnTypeCode, ColumnTypeDeserialize, ColumnTypeSerialize},
};

pub struct ColumnTypeBytes;

impl ColumnType for ColumnTypeBytes {
    fn eq(&self, this: &[u8], that: &[u8]) -> bool {
        this.eq(that)
    }

    fn cmp(&self, this: &[u8], that: &[u8]) -> std::cmp::Ordering {
        this.cmp(that)
    }

    fn code(&self) -> ColumnTypeCode {
        ColumnTypeCode::Bytes
    }
}

impl ColumnTypeDeserialize for ColumnTypeBytes {
    type Output = Box<[u8]>;

    fn deserialize(buf: &[u8]) -> Result<Self::Output, KVRuntimeError> {
        Ok(buf.to_vec().into_boxed_slice())
    }
}

impl ColumnTypeSerialize for ColumnTypeBytes {
    type Input<'a> = &'a [u8];

    fn serialize(input: Self::Input<'_>) -> Result<Box<[u8]>, KVRuntimeError> {
        Ok(Box::from(input))
    }
}
