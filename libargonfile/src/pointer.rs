use std::io::{Read, Write};

use anyhow::{Context, Ok, Result};

use crate::shared::{PositionedWriter, Reader, Writer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pointer {
    offset: u64,
    size: u32,
}

impl Pointer {
    pub fn new(offset: usize, size: usize) -> Self {
        Self {
            offset: offset as u64,
            size: size as u32,
        }
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

pub struct PointerReader;

impl Reader<Pointer> for PointerReader {
    fn try_read<R: Read>(reader: &mut R) -> Result<Pointer> {
        let mut buf = vec![0u8; 12].into_boxed_slice();
        reader
            .read_exact(&mut buf)
            .with_context(|| format!("Failed to read block pointer"))?;

        Ok(Pointer {
            offset: u64::from_be_bytes(buf[0..8].try_into()?),
            size: u32::from_be_bytes(buf[8..12].try_into()?),
        })
    }
}

pub struct PointerWriter;

impl Writer<Pointer> for PointerWriter {
    fn try_write<W: Write>(
        writer: &mut PositionedWriter<W>,
        pointer: &Pointer,
    ) -> Result<crate::pointer::Pointer> {
        let offset = writer.get_position();
        let mut size: usize = 0;

        size += writer.write(&u64::to_be_bytes(pointer.offset))?;
        size += writer.write(&u32::to_be_bytes(pointer.size))?;

        Ok(Pointer::new(offset, size))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::{Ok, Result};

    use crate::shared::{PositionedWriter, Reader, Writer};

    use super::{Pointer, PointerReader, PointerWriter};

    #[test]
    fn test_reader_writer_integration() -> Result<()> {
        // Given
        let mut buf = PositionedWriter::new(vec![]);
        let pointer_to_write = Pointer {
            offset: 0x11223344,
            size: 0xAABBCCDD,
        };

        //When
        PointerWriter::try_write(&mut buf, &pointer_to_write)?;

        let mut buf = Cursor::new(buf.into());
        let pointer_read = PointerReader::try_read(&mut buf)?;

        //Then
        assert_eq!(pointer_read, pointer_to_write);
        Ok(())
    }
}
