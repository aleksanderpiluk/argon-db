use std::io::Write;

use crate::kv::{
    KVRuntimeError, KVRuntimeErrorKind,
    column_type::{ColumnType, ColumnTypeDeserialize, ColumnTypeSerialize},
};

pub struct ColumnTypeU16Array;

impl ColumnType for ColumnTypeU16Array {
    fn eq(&self, this: &[u8], that: &[u8]) -> bool {
        todo!()
    }

    fn cmp(&self, this: &[u8], that: &[u8]) -> std::cmp::Ordering {
        todo!()
    }
}

impl ColumnTypeDeserialize for ColumnTypeU16Array {
    type Output = Box<[u16]>;

    fn deserialize(buf: &[u8]) -> Result<Self::Output, KVRuntimeError> {
        let size_buf = buf[0..8].try_into().map_err(|e| {
            KVRuntimeError::with_msg_and_source(
                KVRuntimeErrorKind::DataMalformed,
                "failed to deserialize u16 array",
                e,
            )
        })?;
        let size = u64::from_le_bytes(size_buf);

        let mut items = Vec::<u16>::with_capacity(size as usize);

        let mut ptr = &buf[8..];

        for _ in 0..size {
            let item_buf = ptr[0..2].try_into().map_err(|e| {
                KVRuntimeError::with_msg_and_source(
                    KVRuntimeErrorKind::DataMalformed,
                    "failed to deserialize u16 array",
                    e,
                )
            })?;

            let item = u16::from_le_bytes(item_buf);
            items.push(item);

            ptr = &ptr[2..];
        }

        Ok(items.into_boxed_slice())
    }
}

impl ColumnTypeSerialize for ColumnTypeU16Array {
    type Input<'a> = &'a [u16];

    fn serialize(input: Self::Input<'_>) -> Result<Box<[u8]>, KVRuntimeError> {
        let size = input.len() as u64;
        let mut buf = Vec::<u8>::new();

        buf.write(&u64::to_le_bytes(size)).unwrap();

        for item in input {
            buf.write(&u16::to_le_bytes(*item)).unwrap();
        }

        Ok(buf.into_boxed_slice())
    }
}
