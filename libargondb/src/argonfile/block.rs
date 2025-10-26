use std::io::{self, Write};

use crate::{
    argonfile::{
        checksum::ArgonfileChecksumStrategy,
        compression::ArgonfileCompressionStrategy,
        error::ArgonfileWriterError,
        utils::{checked_write, inner_writer_error_mapper},
    },
    ensure,
};

pub struct AgronfileBlockWriter<
    W: Write,
    T: ArgonfileChecksumStrategy,
    U: ArgonfileCompressionStrategy,
> {
    block_identifier: BlockIdentifier,

    checksum_strategy: T,
    compression_strategy: U,

    writer: W,
    block_size: usize,
    buffer: Vec<u8>,
}

impl<W: Write, T: ArgonfileChecksumStrategy, U: ArgonfileCompressionStrategy>
    AgronfileBlockWriter<W, T, U>
{
    pub fn new(
        block_identifier: BlockIdentifier,
        checksum_strategy: T,
        compression_strategy: U,
        writer: W,
        block_size: usize,
    ) -> Self {
        Self {
            block_identifier,
            checksum_strategy,
            compression_strategy,
            writer,
            block_size,
            buffer: Vec::with_capacity(block_size),
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, ArgonfileBlockWriterError> {
        let n = self.buffer.write(data)?;

        if n == data.len() {
            Ok(n)
        } else {
            Err(ArgonfileBlockWriterError::PartialWrite(n))
        }
    }

    pub fn end_block(self) -> Result<(), ()> {
        let buffer_size = self.buffer.len();
        ensure!(buffer_size <= u32::MAX as usize, ());

        let checksum = T::calc_checksum(&self.buffer);
        let checksum_size = checksum.len();
        ensure!(checksum_size <= u32::MAX as usize, ());

        let compressed_buffer = U::compress(&self.buffer);
        let compressed_buffer_size = compressed_buffer.len();
        ensure!(compressed_buffer_size <= u32::MAX as usize, ());

        let mut size = 0usize;

        size += BlockHeaderWriter::write(
            &mut self.writer,
            &self.block_identifier,
            compressed_buffer_size as u32,
            buffer_size as u32,
            T::checksum_type(),
            checksum_size as u32,
        )
        .map_err(inner_writer_error_mapper(size))?;

        size += self
            .writer
            .write(&compressed_buffer)
            .map_err(inner_writer_error_mapper(size))?;
        size += self
            .writer
            .write(&checksum)
            .map_err(inner_writer_error_mapper(size))?;
    }
}

pub enum ArgonfileBlockWriterError {
    IOError(io::Error),
    PartialWrite(usize),
}

impl From<io::Error> for ArgonfileBlockWriterError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

struct BlockHeaderWriter;

impl BlockHeaderWriter {
    fn write<W: Write>(
        w: &mut W,
        block_identifier: &BlockIdentifier,
        data_compressed_size: u32,
        data_uncompressed_size: u32,
        checksum_type: u8,
        checksum_size: u32,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut size = 0usize;

        size += checked_write(w, block_identifier).map_err(inner_writer_error_mapper(size))?;

        size += checked_write(w, &u32::to_le_bytes(data_compressed_size))
            .map_err(inner_writer_error_mapper(size))?;
        size += checked_write(w, &u32::to_le_bytes(data_uncompressed_size))
            .map_err(inner_writer_error_mapper(size))?;

        size += checked_write(w, &u8::to_le_bytes(checksum_type))
            .map_err(inner_writer_error_mapper(size))?;
        size += checked_write(w, &[0u8; 3]).map_err(inner_writer_error_mapper(size))?; // Reserved space

        size += checked_write(w, &u32::to_le_bytes(checksum_size))
            .map_err(inner_writer_error_mapper(size))?;

        Ok(size)
    }
}

type BlockIdentifier = [u8; 8];
