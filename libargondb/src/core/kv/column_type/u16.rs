use crate::{
    ensure,
    kv::{
        KVRuntimeError, KVRuntimeErrorKind,
        column_type::{ColumnType, ColumnTypeCode, ColumnTypeDeserialize, ColumnTypeSerialize},
    },
};

pub struct ColumnTypeU16;

impl ColumnType for ColumnTypeU16 {
    fn eq(&self, this: &[u8], that: &[u8]) -> bool {
        this.eq(that)
    }

    fn cmp(&self, this: &[u8], that: &[u8]) -> std::cmp::Ordering {
        this.cmp(that)
    }

    fn code(&self) -> ColumnTypeCode {
        ColumnTypeCode::U16
    }
}

impl ColumnTypeDeserialize for ColumnTypeU16 {
    type Output = u16;

    fn deserialize(buf: &[u8]) -> Result<u16, KVRuntimeError> {
        ensure!(
            buf.len() == 2,
            KVRuntimeError::with_msg(
                KVRuntimeErrorKind::DataMalformed,
                format!("invalid buffer size - expected 2, got: {}", buf.len())
            )
        );

        Ok(u16::from_le_bytes(buf[0..2].try_into().unwrap()))
    }
}

impl ColumnTypeSerialize for ColumnTypeU16 {
    type Input<'a> = u16;

    fn serialize(input: Self::Input<'_>) -> Result<Box<[u8]>, KVRuntimeError> {
        Ok(Box::from(u16::to_le_bytes(input)))
    }
}
