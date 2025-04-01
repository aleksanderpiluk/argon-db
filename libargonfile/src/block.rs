use std::io::{Read, Write};

use anyhow::{anyhow, Context, Ok, Result};
use crc32c::crc32c;

use crate::block_header::{BlockHeader, BlockHeaderReader, BlockHeaderWriter};

const BLOCK_DATA_MAX_SIZE: u32 = 1 << 24;
const CHECKSUM_TYPE_CRC32C: u8 = 0x01;
const CHECKSUM_CRC32C_SIZE: u32 = 0x04;

pub struct Block {
    header: BlockHeader,
    data: Box<[u8]>,
}

pub struct BlockReader;

impl BlockReader {
    pub fn try_read<R: Read>(reader: &mut R) -> Result<Block> {
        let header = BlockHeaderReader::try_read(reader)?;

        let mut data_and_checksum =
            vec![0u8; header.disk_size_without_header as usize].into_boxed_slice();
        reader
            .read_exact(&mut data_and_checksum)
            .with_context(|| format!("Failed to read block data and checksum"))?;

        let data_size = (header.uncompressed_size_without_header - header.checksum_size) as usize;
        let data: Box<[u8]> = data_and_checksum[0..data_size].into();
        let checksum: Box<[u8]> = data_and_checksum[data_size..].into();

        Self::verify_checksum(&header, &data, &checksum)
            .with_context(|| format!("Failed to verify checksum"))?;

        Ok(Block { header, data })
    }

    fn verify_checksum(header: &BlockHeader, data: &Box<[u8]>, checksum: &Box<[u8]>) -> Result<()> {
        if header.checksum_type != CHECKSUM_TYPE_CRC32C {
            return Err(anyhow!("Invalid checksum type"));
        }

        let checksum_bytes: [u8; 4] = checksum[0..4]
            .try_into()
            .with_context(|| format!("Failed to read CRC32C checksum"))?;
        let checksum = u32::from_be_bytes(checksum_bytes);
        let checksum_calc = crc32c(data);

        if checksum != checksum_calc {
            return Err(anyhow!("Checksum do not match"));
        }
        Ok(())
    }
}

pub struct BlockWriter;

impl BlockWriter {
    pub fn try_write<W: Write>(writer: &mut W, identifier: u64, block_data: &[u8]) -> Result<()> {
        let data_size = block_data.len();
        if data_size > BLOCK_DATA_MAX_SIZE as usize {
            return Err(anyhow!("Block data size exceeds allowed size"));
        }

        let data_size = data_size as u32;

        let checksum = crc32c(block_data);
        let uncompressed_size = data_size + 4;

        let header = BlockHeader {
            identifier,
            disk_size_without_header: uncompressed_size,
            uncompressed_size_without_header: uncompressed_size,
            checksum_type: CHECKSUM_TYPE_CRC32C,
            checksum_size: CHECKSUM_CRC32C_SIZE,
        };
        BlockHeaderWriter::try_write(writer, &header)?;

        writer.write(block_data)?;
        writer.write(&u32::to_be_bytes(checksum))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};

    use anyhow::{Ok, Result};

    use super::{BlockReader, BlockWriter};

    #[test]
    fn test_reader_writer_integration() -> Result<()> {
        //Given
        let buffer: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(buffer);
        let data_to_write = String::from("Lorem ipsum dolor sit amet");
        let identifier = 0xABCDEF0FEDCDA;

        // When
        BlockWriter::try_write(&mut cursor, identifier, data_to_write.as_bytes())?;
        cursor.seek(SeekFrom::Start(0))?;
        let block = BlockReader::try_read(&mut cursor)?;

        //Then
        assert_eq!(block.header.identifier, identifier);
        assert_eq!(block.data.as_ref(), data_to_write.as_bytes());
        Ok(())
    }
}
