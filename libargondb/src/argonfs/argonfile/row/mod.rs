//! Row stores a group of mutations sharing same primary key.

use super::{
    error::{ArgonfileBuilderError, ArgonfileWriterError},
    utils::{
        ArgonfileOffsetCountingWriteWrapper, ArgonfileSizeCountingWriter, ArgonfileWrite,
        checked_write, inner_writer_error_mapper,
    },
};
use crate::{
    ensure,
    kv::{mutation::KVMutation, primary_key::KVPrimaryKeyUtils},
};
use std::io::{self, Write};

pub struct ArgonfileRowBuilder {
    primary_key: Box<[u8]>,
    buffer: ArgonfileOffsetCountingWriteWrapper<Vec<u8>>,
}

impl ArgonfileRowBuilder {
    pub fn new(primary_key: Box<[u8]>) -> Self {
        Self {
            primary_key,
            buffer: ArgonfileOffsetCountingWriteWrapper::new(vec![]),
        }
    }

    pub fn belongs_to_row(&self, mutation: &impl KVMutation) -> bool {
        let row_pk = &self.primary_key[..];
        let mutation_pk = mutation.primary_key();

        return row_pk.eq(mutation_pk);
    }

    pub fn add_mutation(
        &mut self,
        mutation: &impl KVMutation,
    ) -> Result<usize, ArgonfileBuilderError> {
        ensure!(
            self.belongs_to_row(mutation),
            ArgonfileBuilderError::AssertionError
        );

        Ok(RowMutationWriter::write(&mut self.buffer, mutation)?)
    }

    pub fn end_row(self, w: &mut impl ArgonfileWrite) -> Result<usize, ArgonfileWriterError> {
        let buffer = self.buffer.into_inner();
        ArgonfileRowWriter::write(w, &self.primary_key, &buffer)
    }
}

struct ArgonfileRowWriter;

impl ArgonfileRowWriter {
    fn write(
        w: &mut impl ArgonfileWrite,
        primary_key: &[u8],
        mutations: &[u8],
    ) -> Result<usize, ArgonfileWriterError> {
        let primary_key_size = KVPrimaryKeyUtils::size(primary_key);
        let mutations_size = mutations.len();
        assert!(mutations_size <= u32::MAX as usize);

        let mut writer = ArgonfileSizeCountingWriter::new(w);

        writer.write(&u16::to_le_bytes(primary_key_size))?;
        writer.write(&u32::to_le_bytes(mutations_size as u32))?;
        writer.write(primary_key)?;
        writer.write(mutations)?;

        Ok(writer.size())
    }
}

struct RowMutationWriter;

impl RowMutationWriter {
    fn write(
        writer: &mut impl ArgonfileWrite,
        mutation: &impl KVMutation,
    ) -> Result<usize, ArgonfileWriterError> {
        // TODO: ERROR HANDLING
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        writer.write(&u64::to_le_bytes(mutation.timestamp()))?;
        writer.write(&u16::to_le_bytes(mutation.column_id()))?;
        writer.write(&u8::to_le_bytes(mutation.mutation_type() as u8))?;
        writer.write(&u64::to_le_bytes(mutation.value_size()))?;
        writer.write(mutation.value())?;

        Ok(writer.size())
    }
}
