use std::io::{Read, Write};

use anyhow::{Context, Ok, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPointer {
    offset: u64,
    size: u32,
}

pub struct BlockPointerReader;

impl BlockPointerReader {
    pub fn try_read<R: Read>(reader: &mut R) -> Result<BlockPointer> {
        let mut buf = vec![0u8; 12].into_boxed_slice();
        reader
            .read_exact(&mut buf)
            .with_context(|| format!("Failed to read block pointer"))?;

        Ok(BlockPointer {
            offset: u64::from_be_bytes(buf[0..8].try_into()?),
            size: u32::from_be_bytes(buf[8..12].try_into()?),
        })
    }
}

pub struct BlockPointerWriter;

impl BlockPointerWriter {
    pub fn try_write<W: Write>(writer: &mut W, pointer: &BlockPointer) -> Result<()> {
        writer.write(&u64::to_be_bytes(pointer.offset))?;
        writer.write(&u32::to_be_bytes(pointer.size))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::{Ok, Result};

    use super::{BlockPointer, BlockPointerReader, BlockPointerWriter};

    #[test]
    fn test_reader_writer_integration() -> Result<()> {
        // Given
        let mut cursor: Cursor<Vec<u8>> = Cursor::default();
        let pointer_to_write = BlockPointer {
            offset: 0x11223344,
            size: 0xAABBCCDD,
        };

        //When
        BlockPointerWriter::try_write(&mut cursor, &pointer_to_write)?;
        cursor.set_position(0);
        let pointer_read = BlockPointerReader::try_read(&mut cursor)?;

        //Then
        assert_eq!(pointer_read, pointer_to_write);
        Ok(())
    }
}
