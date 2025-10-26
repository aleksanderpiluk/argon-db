//! Row stores a group of mutations sharing same primary key.

use std::io::{self, Write};

use crate::{
    argonfile::{
        error::ArgonfileWriterError,
        utils::{checked_write, inner_writer_error_mapper},
    },
    kv::{mutation::KVMutation, primary_key::KVPrimaryKeyUtils},
};

pub struct ArgonfileRowBuilder {
    primary_key: Box<[u8]>,
    buffer: Vec<u8>,
}

impl ArgonfileRowBuilder {
    pub fn new(primary_key: Box<[u8]>) -> Self {
        Self {
            primary_key,
            buffer: vec![],
        }
    }

    pub fn belongs_to_row(&self, mutation: &impl KVMutation) -> bool {
        let row_pk = &self.primary_key[..];
        let mutation_pk = mutation.primary_key();

        return row_pk.eq(mutation_pk);
    }

    pub fn add_mutation(&mut self, mutation: &impl KVMutation) {
        assert!(self.belongs_to_row(mutation));

        RowMutationWriter::write(&mut self.buffer, mutation).unwrap()
    }

    pub fn end_row<W: Write>(self, w: &mut W) -> Result<usize, ArgonfileWriterError> {
        ArgonfileRowWriter::write(w, &self.primary_key, &self.buffer)
    }
}

struct ArgonfileRowWriter;

impl ArgonfileRowWriter {
    fn write<W: Write>(
        w: &mut W,
        primary_key: &[u8],
        mutations: &[u8],
    ) -> Result<usize, ArgonfileWriterError> {
        let primary_key_size = KVPrimaryKeyUtils::size(primary_key);
        let mutations_size = mutations.len();
        assert!(mutations_size <= u32::MAX as usize);

        let mut size = 0usize;
        size += checked_write(w, &u16::to_le_bytes(primary_key_size))
            .map_err(inner_writer_error_mapper(size))?;
        size += checked_write(w, &u32::to_le_bytes(mutations_size as u32))
            .map_err(inner_writer_error_mapper(size))?;
        size += checked_write(w, primary_key).map_err(inner_writer_error_mapper(size))?;
        size += checked_write(w, mutations).map_err(inner_writer_error_mapper(size))?;

        Ok(size)
    }
}

struct RowMutationWriter;

impl RowMutationWriter {
    fn write<W: Write>(writer: &mut W, mutation: &impl KVMutation) -> Result<(), io::Error> {
        // TODO: ERROR HANDLING
        let size = writer.write(&u64::to_le_bytes(mutation.timestamp()))?;
        writer.write(&u16::to_le_bytes(mutation.column_id()))?;
        writer.write(&u8::to_le_bytes(mutation.mutation_type() as u8))?;
        writer.write(&u64::to_le_bytes(mutation.value_size()))?;
        writer.write(mutation.value())?;

        Ok(())
    }
}
