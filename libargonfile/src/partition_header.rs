use std::io::{Read, Write};

use anyhow::{anyhow, Context, Ok, Result};

const FLAG_HAS_TIMESTAMP: u16 = 0x01;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionHeader {
    flags: u16,
    timestamp: Option<u64>,
    key: PartitionHeaderKey,
}

impl PartitionHeader {
    pub fn with_key(key: PartitionHeaderKey) -> Self {
        Self {
            flags: 0x0,
            timestamp: None,
            key,
        }
    }

    pub fn with_timestamp_and_key(timestamp: u64, key: PartitionHeaderKey) -> Self {
        Self {
            flags: FLAG_HAS_TIMESTAMP,
            timestamp: Some(timestamp),
            key,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionHeaderKey(Box<[u8]>);

impl PartitionHeaderKey {
    pub fn len(&self) -> u16 {
        self.0.len() as u16
    }
}

impl TryFrom<Box<[u8]>> for PartitionHeaderKey {
    type Error = anyhow::Error;

    fn try_from(value: Box<[u8]>) -> std::result::Result<Self, Self::Error> {
        let key_size = value.len();
        if key_size > u16::MAX as usize {
            return Err(anyhow!("Key exceeds max allowed size for partition key"));
        }

        Ok(Self(value))
    }
}

impl Into<Box<[u8]>> for PartitionHeaderKey {
    fn into(self) -> Box<[u8]> {
        self.0
    }
}

// impl AsRef<Box<[u8]>> for PartitionHeaderKey {
//     fn as_ref(&self) -> &Box<[u8]> {
//         &self.0
//     }
// }

pub struct PartitionHeaderReader;

impl PartitionHeaderReader {
    pub fn try_read<R: Read>(reader: &mut R) -> Result<PartitionHeader> {
        let mut flags_and_key_size = vec![0u8; 4].into_boxed_slice();
        reader
            .read_exact(&mut flags_and_key_size)
            .with_context(|| format!("Failed to read partition header"))?;

        let flags = u16::from_be_bytes(flags_and_key_size[0..2].try_into()?);
        let key_size = u16::from_be_bytes(flags_and_key_size[2..4].try_into()?);

        let has_timestamp = (flags & FLAG_HAS_TIMESTAMP) != 0;
        let timestamp = if has_timestamp {
            let mut timestamp = vec![0u8; 8].into_boxed_slice();
            reader
                .read_exact(&mut timestamp)
                .with_context(|| format!("Failed to read partition header timestamp"))?;
            Some(u64::from_be_bytes(timestamp[0..8].try_into()?))
        } else {
            None
        };

        let mut key = vec![0u8; key_size as usize].into_boxed_slice();
        if key_size > 0 {
            reader
                .read_exact(&mut key)
                .with_context(|| format!("Failed to read partition header key"))?;
        }

        Ok(PartitionHeader {
            flags,
            timestamp,
            key: key.try_into()?,
        })
    }
}

pub struct PartitionHeaderWriter;

impl PartitionHeaderWriter {
    pub fn try_write<W: Write>(writer: &mut W, header: &PartitionHeader) -> Result<()> {
        writer.write(&u16::to_be_bytes(header.flags))?;

        writer.write(&u16::to_be_bytes(header.key.len()))?;

        if let Some(timestamp) = header.timestamp {
            writer.write(&u64::to_be_bytes(timestamp))?;
        }

        writer.write(&header.key.0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::{Ok, Result};

    use super::{
        PartitionHeader, PartitionHeaderKey, PartitionHeaderReader, PartitionHeaderWriter,
    };

    #[test]
    fn test_reader_writer_integration() -> Result<()> {
        //Given
        let mut cursor: Cursor<Vec<u8>> = Cursor::default();
        let key = PartitionHeaderKey::try_from(Box::from(String::from("some_key").as_bytes()))?;
        let header_to_write = PartitionHeader::with_timestamp_and_key(0x11223344, key);

        //When
        PartitionHeaderWriter::try_write(&mut cursor, &header_to_write)?;
        cursor.set_position(0);
        let header_read = PartitionHeaderReader::try_read(&mut cursor)?;

        //Then
        assert_eq!(header_read, header_to_write);
        Ok(())
    }
}
