use std::io::Write;

use crate::{
    argonfs::argonfile::{
        block::{
            BlockPointer, block::ArgonfileBlockWriterError, block_header::BlockHeader,
            block_identifier::BlockIdentifier, checksum::ChecksumAlgo,
            compression::CompressionAlgo,
        },
        error::{ArgonfileBuilderError, ArgonfileWriterError},
        utils::{ArgonfileSizeCountingWriter, ArgonfileWrite, checked_write},
    },
    ensure,
};

pub struct BlockBuilder {
    desired_block_size: usize,
    buffer: Vec<u8>,
}

impl BlockBuilder {
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
    ) -> Result<BlockPointer, ArgonfileBuilderError> {
        let buffer_size = self.buffer.len();
        ensure!(
            buffer_size <= u32::MAX as usize,
            ArgonfileBuilderError::from_msg("AssertionError ?")
        );

        let mut checksum = Vec::<u8>::new();
        let checksum_size = checksum_algo
            .calc_checksum(&self.buffer, &mut checksum)
            .unwrap();
        ensure!(
            checksum_size <= u32::MAX as usize,
            ArgonfileBuilderError::from_msg("AssertionError ?")
        );

        let mut compressed_buffer = Vec::<u8>::new();
        compression_algo
            .compress(&self.buffer, &mut compressed_buffer)
            .unwrap();
        let compressed_buffer_size = compressed_buffer.len();
        ensure!(
            compressed_buffer_size <= u32::MAX as usize,
            ArgonfileBuilderError::from_msg("AssertionError ?")
        );

        let offset = writer.offset();
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        BlockHeader::serialize(
            &mut writer,
            &block_identifier,
            compressed_buffer_size as u32,
            buffer_size as u32,
            checksum_algo.checksum_type(),
            checksum_size as u32,
            compression_algo.compression_type(),
        )?;

        writer.write(&compressed_buffer)?;
        writer.write(&checksum)?;

        let size = writer.size();
        ensure!(
            size <= u32::MAX as usize,
            ArgonfileBuilderError::from_msg("AssertionError ?")
        );

        Ok(BlockPointer::new(offset as u64, size as u32))
    }
}

impl ArgonfileWrite for BlockBuilder {
    fn offset(&self) -> usize {
        panic!("This function should not be called, as block is compressed")
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ArgonfileWriterError> {
        checked_write(&mut self.buffer, buf)
    }
}
