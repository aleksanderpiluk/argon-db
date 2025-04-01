use std::io::{Read, Write};

use anyhow::{Context, Ok, Result};

#[derive(Debug, PartialEq, Eq)]
pub struct BlockHeader {
    pub identifier: u64,
    pub disk_size_without_header: u32,
    pub uncompressed_size_without_header: u32,
    pub checksum_type: u8,
    pub checksum_size: u32,
}

pub struct BlockHeaderReader;

impl BlockHeaderReader {
    pub fn try_read<R: Read>(reader: &mut R) -> Result<BlockHeader> {
        let mut buf = [0u8; 24];

        reader
            .read_exact(&mut buf)
            .with_context(|| format!("Failed to read block header"))?;

        Ok(BlockHeader {
            identifier: u64::from_be_bytes(buf[0..8].try_into().unwrap()),
            disk_size_without_header: u32::from_be_bytes(buf[8..12].try_into().unwrap()),
            uncompressed_size_without_header: u32::from_be_bytes(buf[12..16].try_into().unwrap()),
            checksum_type: u8::from_be_bytes(buf[16..17].try_into().unwrap()),
            checksum_size: u32::from_be_bytes(buf[20..24].try_into().unwrap()),
        })
    }
}

pub struct BlockHeaderWriter;

impl BlockHeaderWriter {
    pub fn try_write<W: Write>(writer: &mut W, header: &BlockHeader) -> Result<()> {
        || -> Result<()> {
            writer.write(&u64::to_be_bytes(header.identifier))?;
            writer.write(&u32::to_be_bytes(header.disk_size_without_header))?;
            writer.write(&u32::to_be_bytes(header.uncompressed_size_without_header))?;
            writer.write(&u8::to_be_bytes(header.checksum_type))?;
            writer.write(&vec![0u8; 3])?;
            writer.write(&u32::to_be_bytes(header.checksum_size))?;
            Ok(())
        }()
        .with_context(|| format!("Failed to write block header"))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek};

    use anyhow::Result;

    use super::{BlockHeader, BlockHeaderReader, BlockHeaderWriter};

    #[test]
    fn test_reader_writer_integration() -> Result<()> {
        //Given
        let buffer: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(buffer);
        let header_to_write = BlockHeader {
            identifier: 0x123456789ABCDEF0,
            disk_size_without_header: 0x1234,
            uncompressed_size_without_header: 0xFEDC,
            checksum_type: 0x01,
            checksum_size: 0x04,
        };

        // When
        BlockHeaderWriter::try_write(&mut cursor, &header_to_write)?;
        cursor.seek(std::io::SeekFrom::Start(0))?;
        let header_read = BlockHeaderReader::try_read(&mut cursor)?;

        // Then
        assert_eq!(header_to_write, header_read);
        Ok(())
    }
}
