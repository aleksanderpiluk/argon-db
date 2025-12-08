use std::io::{self, Cursor, Write};

use crate::{
    argonfs::argonfile::{
        block_header::{BlockHeader, BlockHeaderReader, BlockHeaderWriter},
        block_identifier::BlockIdentifier,
        block_ptr::ArgonfileBlockPointer,
        checksum::{ChecksumAlgo, ChecksumAlgoResolver},
        compression::{CompressionAlgo, CompressionAlgoResolver},
        error::{ArgonfileBuilderError, ArgonfileWriterError},
        utils::{ArgonfileSizeCountingWriter, ArgonfileWrite, checked_write},
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

    pub fn build(
        self,
        writer: &mut impl ArgonfileWrite,
        block_identifier: &BlockIdentifier,
        checksum_algo: &impl ChecksumAlgo,
        compression_algo: &impl CompressionAlgo,
    ) -> Result<ArgonfileBlockPointer, ArgonfileBuilderError> {
        let buffer_size = self.buffer.len();
        ensure!(
            buffer_size <= u32::MAX as usize,
            ArgonfileBuilderError::AssertionError
        );

        let mut checksum = Vec::<u8>::new();
        let checksum_size = checksum_algo
            .calc_checksum(&self.buffer, &mut checksum)
            .unwrap();
        ensure!(
            checksum_size <= u32::MAX as usize,
            ArgonfileBuilderError::AssertionError
        );

        let mut compressed_buffer = Vec::<u8>::new();
        compression_algo
            .compress(&self.buffer, &mut compressed_buffer)
            .unwrap();
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
            checksum_algo.checksum_type(),
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

pub struct BlockReader;

impl BlockReader {
    pub fn read(&self, buf: &[u8]) -> Result<(), ()> {
        let buf_header_end_idx = BlockHeader::SIZE_SERIALIZED;
        let buf_header = &buf[0..buf_header_end_idx];

        let header = BlockHeaderReader::read(buf_header).map_err(|_| ())?;

        let compressed_size = header.data_compressed_size as usize;
        let buf_compressed_end_idx = buf_header_end_idx + compressed_size;
        let buf_compressed = &buf[buf_header_end_idx..buf_compressed_end_idx];

        let checksum_size = header.checksum_size as usize;
        let buf_checksum_end_idx = buf_compressed_end_idx + checksum_size;
        let buf_checksum = &buf[buf_compressed_end_idx..(buf_checksum_end_idx)];

        let compression_type = header.compression_type;
        let compression_algo = CompressionAlgoResolver::for_compression_type(compression_type);

        let decompressed_size = header.data_uncompressed_size as usize;
        let buf_decompressed = vec![0u8; decompressed_size].into_boxed_slice();

        let mut buf_decompressed_writer = Cursor::new(buf_decompressed);
        compression_algo
            .decompress(buf_compressed, &mut buf_decompressed_writer)
            .map_err(|_| ())?;
        let buf_decompressed = buf_decompressed_writer.into_inner();

        let checksum_type = header.checksum_type;
        let checksum_algo = ChecksumAlgoResolver::for_checksum_type(checksum_type);

        checksum_algo
            .verify_checksum(&buf_decompressed, buf_checksum)
            .map_err(|_| ())?;

        Ok(())
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
