use crate::kv::{
    KVRuntimeError, KVRuntimeErrorKind,
    column_type::{ColumnType, ColumnTypeCode, ColumnTypeDeserialize, ColumnTypeSerialize},
};

pub struct ColumnTypeText;

impl ColumnType for ColumnTypeText {
    fn eq(&self, this: &[u8], that: &[u8]) -> bool {
        this.eq(that)
    }

    fn cmp(&self, this: &[u8], that: &[u8]) -> std::cmp::Ordering {
        this.cmp(that)
    }

    fn code(&self) -> ColumnTypeCode {
        ColumnTypeCode::Text
    }
}

impl ColumnTypeDeserialize for ColumnTypeText {
    type Output = String;

    fn deserialize(buf: &[u8]) -> Result<String, KVRuntimeError> {
        String::from_utf8(buf.to_owned()).map_err(|e| {
            KVRuntimeError::with_msg_and_source(
                KVRuntimeErrorKind::DataMalformed,
                "cannot deserialize to text",
                e,
            )
        })
    }
}

impl ColumnTypeSerialize for ColumnTypeText {
    type Input<'a> = &'a str;

    fn serialize(input: Self::Input<'_>) -> Result<Box<[u8]>, KVRuntimeError> {
        Ok(Box::from(input.as_bytes()))
    }
}
