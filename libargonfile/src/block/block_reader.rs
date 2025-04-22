use std::io::Read;

use anyhow::{anyhow, Result};
use crc32c::crc32c;

use super::{Block, BlockIdentifier, CHECKSUM_TYPE_CRC32C};

pub struct BlockReader;

impl BlockReader {
    pub fn try_read<R: Read>(reader: &mut R) -> Result<Block> {
        let mut buf = [0u8; 24];

        reader.read_exact(&mut buf)?;

        let block_identifier = BlockIdentifier::try_from(&buf[0..8]).unwrap();
        let disk_size_without_header = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        let uncompressed_size_without_header = u32::from_be_bytes(buf[12..16].try_into().unwrap());
        let checksum_type = u8::from_be_bytes(buf[16..17].try_into().unwrap());
        let checksum_size = u32::from_be_bytes(buf[20..24].try_into().unwrap());

        let mut data_and_checksum = vec![0u8; disk_size_without_header as usize].into_boxed_slice();
        reader.read_exact(&mut data_and_checksum)?;

        // TODO: compression support

        let data_size = (uncompressed_size_without_header - checksum_size) as usize;
        let data: Box<[u8]> = data_and_checksum[0..data_size].into();
        let checksum: Box<[u8]> = data_and_checksum[data_size..].into();

        Self::verify_checksum(checksum_type, &data, &checksum)?;

        Ok(Block {
            identifier: block_identifier,
            disk_size_without_header,
            uncompressed_size_without_header,
            checksum_type,
            checksum_size,
            data,
            checksum,
        })
    }

    fn verify_checksum(checksum_type: u8, data: &Box<[u8]>, checksum: &Box<[u8]>) -> Result<()> {
        if checksum_type != CHECKSUM_TYPE_CRC32C {
            return Err(anyhow!("Invalid checksum type"));
        }

        let checksum_bytes: [u8; 4] = checksum[0..4].try_into()?;
        let checksum = u32::from_be_bytes(checksum_bytes);
        let checksum_calc = crc32c(data);

        if checksum != checksum_calc {
            return Err(anyhow!("Checksum do not match"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::block::{
        block_reader::BlockReader, Block, BlockIdentifier, CHECKSUM_CRC32C_SIZE,
        CHECKSUM_TYPE_CRC32C,
    };

    #[test]
    fn test_reader() {
        // given
        let mut reader =
             Cursor::new(b"BLK_DATA\x00\x00\x00\x11\x00\x00\x00\x11\x01\x00\x00\x00\x00\x00\x00\x04Hello, world!\xc8\xa1\x06\xe5");

        // when
        let block = BlockReader::try_read(&mut reader).unwrap();

        // then
        assert_eq!(
            block,
            Block {
                identifier: BlockIdentifier::DATA_BLOCK,
                disk_size_without_header: 17,
                uncompressed_size_without_header: 17,
                checksum_type: CHECKSUM_TYPE_CRC32C,
                checksum_size: CHECKSUM_CRC32C_SIZE,
                checksum: Box::from(b"\xc8\xa1\x06\xe5".as_slice()),
                data: Box::from(b"Hello, world!".as_slice()),
            }
        );
    }
}
