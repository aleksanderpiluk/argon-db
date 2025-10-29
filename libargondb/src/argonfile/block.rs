use std::io::{self, Write};

use crate::{
    argonfile::{
        block_identifier::BlockIdentifier,
        block_ptr::ArgonfileBlockPointer,
        checksum::ArgonfileChecksumStrategy,
        compression::ArgonfileCompressionStrategy,
        error::{ArgonfileBuilderError, ArgonfileWriterError},
        utils::{
            ArgonfileSizeCountingWriter, ArgonfileWrite, checked_write, inner_writer_error_mapper,
        },
    },
    ensure,
};

pub struct ArgonfileBlockBuilder {
    desired_block_size: usize,
    buffer: Vec<u8>,
}

impl ArgonfileBlockBuilder {
    pub fn new(desired_block_size: usize) -> Self {
        Self {
            desired_block_size,
            buffer: Vec::with_capacity(desired_block_size),
        }
    }

    pub fn is_desired_size_exceeded(&self) -> bool {
        self.buffer.len() >= self.desired_block_size
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, ArgonfileBlockWriterError> {
        let n = self.buffer.write(data)?;

        if n == data.len() {
            Ok(n)
        } else {
            Err(ArgonfileBlockWriterError::PartialWrite(n))
        }
    }

    pub fn build<T: ArgonfileChecksumStrategy, U: ArgonfileCompressionStrategy>(
        self,
        writer: &mut impl ArgonfileWrite,
        block_identifier: &BlockIdentifier,
        checksum_strategy: &T,
        compression_strategy: &U,
    ) -> Result<ArgonfileBlockPointer, ArgonfileBuilderError> {
        let buffer_size = self.buffer.len();
        ensure!(
            buffer_size <= u32::MAX as usize,
            ArgonfileBuilderError::AssertionError
        );

        let checksum_type = checksum_strategy.checksum_type();
        let checksum = T::calc_checksum(&self.buffer);
        let checksum_size = checksum.len();
        ensure!(
            checksum_size <= u32::MAX as usize,
            ArgonfileBuilderError::AssertionError
        );

        let compressed_buffer = U::compress(&self.buffer);
        let compressed_buffer_size = compressed_buffer.len();
        ensure!(
            compressed_buffer_size <= u32::MAX as usize,
            ArgonfileBuilderError::AssertionError
        );

        let offset = writer.offset();
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        BlockHeaderWriter::write(
            &mut writer,
            &block_identifier,
            compressed_buffer_size as u32,
            buffer_size as u32,
            checksum_type,
            checksum_size as u32,
        )?;

        writer.write(&compressed_buffer)?;
        writer.write(&checksum)?;

        let size = writer.size();
        ensure!(
            size <= u32::MAX as usize,
            ArgonfileBuilderError::AssertionError
        );

        Ok(ArgonfileBlockPointer::new(offset as u64, size as u32))
    }
}

impl ArgonfileWrite for ArgonfileBlockBuilder {
    fn offset(&self) -> usize {
        panic!("This function should not be called, as block is compressed")
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ArgonfileWriterError> {
        checked_write(&mut self.buffer, buf)
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
    fn write(
        writer: &mut impl ArgonfileWrite,
        block_identifier: &BlockIdentifier,
        data_compressed_size: u32,
        data_uncompressed_size: u32,
        checksum_type: u8,
        checksum_size: u32,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        writer.write(block_identifier)?;
        writer.write(&u32::to_le_bytes(data_compressed_size))?;
        writer.write(&u32::to_le_bytes(data_uncompressed_size))?;
        writer.write(&u8::to_le_bytes(checksum_type))?;
        writer.write(&[0u8; 3])?; // Reserved space
        writer.write(&u32::to_le_bytes(checksum_size))?;

        Ok(writer.size())
    }
}
