use std::{
    array::TryFromSliceError,
    io::{self, Write},
};

use crate::{
    argonfs::{
        argonfile::{
            block_identifier::BlockIdentifier,
            block_ptr::ArgonfileBlockPointer,
            checksum::{ArgonfileChecksumStrategy, ArgonfileChecksumStrategyFactory},
            compression::ArgonfileCompressionStrategy,
            error::{ArgonfileBuilderError, ArgonfileWriterError},
            utils::{ArgonfileSizeCountingWriter, ArgonfileWrite, checked_write},
        },
        buffer_allocator::BufferAllocator,
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

pub struct ArgonfileBlockReader<C: ArgonfileCompressionStrategy> {
    compression: C,
}

impl<C: ArgonfileCompressionStrategy> ArgonfileBlockReader<C> {
    pub fn new(compression: C) -> Self {
        Self { compression }
    }
}

impl<C: ArgonfileCompressionStrategy> ArgonfileBlockReader<C> {
    pub fn read(&self, buf: &[u8], mut allocator: impl BufferAllocator) {
        let header_bytes = <[u8; 24]>::try_from(&buf[0..24]).unwrap();
        let header = BlockHeaderReader::read(&header_bytes).unwrap();

        let compressed_buffer_end = 24 + header.data_compressed_size as usize;
        let compressed_buffer = &buf[24..compressed_buffer_end];

        let checksum_buffer_end = compressed_buffer_end + header.checksum_size as usize;
        let checksum = &buf[compressed_buffer_end..checksum_buffer_end];

        let mut out_buffer = allocator.alloc(header.data_uncompressed_size as _);
        self.compression
            .decompress(compressed_buffer, &mut out_buffer);

        let checksum_stategy =
            ArgonfileChecksumStrategyFactory::from_checksum_type(header.checksum_type);
        assert!(
            checksum_stategy
                .verify_checksum(&mut out_buffer, checksum)
                .unwrap()
                == true
        );
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

pub struct BlockHeader {
    pub block_identifier: BlockIdentifier,
    pub data_compressed_size: u32,
    pub data_uncompressed_size: u32,
    pub checksum_type: u8,
    pub checksum_size: u32,
}

pub struct BlockHeaderReader;

impl BlockHeaderReader {
    pub fn read(buf: &[u8; 24]) -> Result<BlockHeader, TryFromSliceError> {
        let block_identifier: [u8; 8] = <[u8; 8]>::try_from(&buf[0..8])?;
        let data_compressed_size = u32::from_le_bytes(<[u8; 4]>::try_from(&buf[8..12])?);
        let data_uncompressed_size = u32::from_le_bytes(<[u8; 4]>::try_from(&buf[12..16])?);
        let checksum_type = u8::from_le_bytes(<[u8; 1]>::try_from(&buf[16..17])?);
        let checksum_size = u32::from_le_bytes(<[u8; 4]>::try_from(&buf[20..24])?);

        Ok(BlockHeader {
            block_identifier,
            data_compressed_size,
            data_uncompressed_size,
            checksum_type,
            checksum_size,
        })
    }
}
