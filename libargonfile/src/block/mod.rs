mod block_builder;
mod block_reader;
mod block_writer;

use anyhow::{anyhow, Result};
pub use block_builder::BlockBuilder;
pub use block_writer::BlockWriter;
use crc32c::crc32c;

const CHECKSUM_TYPE_CRC32C: u8 = 0x01;
const CHECKSUM_CRC32C_SIZE: u32 = 0x04;

#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    identifier: BlockIdentifier,
    pub disk_size_without_header: u32,
    pub uncompressed_size_without_header: u32,
    pub checksum_type: u8,
    pub checksum_size: u32,
    data: Box<[u8]>,
    checksum: Box<[u8]>,
}

impl Block {
    pub const BLOCK_DATA_MAX_SIZE: u32 = 1 << 24;
    pub const BLOCK_CONTENT_SIZE: u32 = 1 << 16;

    pub fn new(identifier: BlockIdentifier, block_data: Box<[u8]>) -> Result<Block> {
        let data_size = block_data.len();
        if data_size > Self::BLOCK_DATA_MAX_SIZE as usize {
            return Err(anyhow!("Block data size exceeds allowed size"));
        }

        let data_size = data_size as u32;

        let checksum = crc32c(&block_data);
        let uncompressed_size = data_size + 4;

        Ok(Self {
            identifier,
            disk_size_without_header: uncompressed_size,
            uncompressed_size_without_header: uncompressed_size,
            checksum_type: CHECKSUM_TYPE_CRC32C,
            checksum_size: CHECKSUM_CRC32C_SIZE,
            data: block_data,
            checksum: Box::new(u32::to_be_bytes(checksum)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockIdentifier([u8; 8]);

impl BlockIdentifier {
    pub const DATA_BLOCK: BlockIdentifier = BlockIdentifier(*b"BLK_DATA");
    pub const INDEX_BLOCK: BlockIdentifier = BlockIdentifier(*b"BLK_INDX");
    pub const SUMMARY_BLOCK: BlockIdentifier = BlockIdentifier(*b"BLK_SUMM");
}

impl TryFrom<&[u8]> for BlockIdentifier {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let identifier: [u8; 8] = value.try_into()?;

        Ok(BlockIdentifier(identifier))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Cursor, Seek, SeekFrom},
        vec,
    };

    use anyhow::{Ok, Result};

    use crate::{
        block::{block_reader::BlockReader, Block, BlockIdentifier, BlockWriter},
        shared::{PositionedWriter, Writer},
    };

    // use super::{BlockReader, BlockWriter};

    #[test]
    fn test_reader_writer_integration() {
        //Given
        let mut buf = PositionedWriter::new(vec![]);
        let block_data: Box<[u8]> = Box::from(b"Lorem ipsum dolor sit amet".as_slice());
        let block = Block::new(BlockIdentifier::DATA_BLOCK, block_data).unwrap();

        // When
        BlockWriter::try_write(&mut buf, &block).unwrap();

        let mut buf = Cursor::new(buf.into());
        let read_block = BlockReader::try_read(&mut buf).unwrap();

        //Then
        assert_eq!(block, read_block);
    }
}
