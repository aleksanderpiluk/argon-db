use std::io::Write;

use anyhow::Ok;

use crate::{pointer::Pointer, shared::Writer};

use super::Block;

pub struct BlockWriter;

impl Writer<Block> for BlockWriter {
    fn try_write<W: std::io::Write>(
        writer: &mut crate::shared::PositionedWriter<W>,
        block: &Block,
    ) -> anyhow::Result<crate::pointer::Pointer> {
        let offset = writer.get_position();
        let mut size: usize = 0;

        size += writer.write(block.identifier.0.as_slice())?;
        size += writer.write(&u32::to_be_bytes(block.disk_size_without_header))?;
        size += writer.write(&u32::to_be_bytes(block.uncompressed_size_without_header))?;
        size += writer.write(&u8::to_be_bytes(block.checksum_type))?;
        size += writer.write(&vec![0u8; 3])?;
        size += writer.write(&u32::to_be_bytes(block.checksum_size))?;
        size += writer.write(&block.data)?;
        size += writer.write(&block.checksum)?;

        Ok(Pointer::new(offset, size))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block::{Block, BlockIdentifier},
        shared::{PositionedWriter, Writer},
    };

    use super::BlockWriter;

    #[test]
    fn test_writer() {
        // given
        let block_data: Box<[u8]> = Box::from(b"Hello, world!".as_slice());
        let block = Block::new(BlockIdentifier::DATA_BLOCK, block_data).unwrap();
        let mut writer = PositionedWriter::new(vec![0u8; 0]);
        let expected_out =
            b"BLK_DATA\x00\x00\x00\x11\x00\x00\x00\x11\x01\x00\x00\x00\x00\x00\x00\x04Hello, world!\xc8\xa1\x06\xe5";

        // when
        BlockWriter::try_write(&mut writer, &block).unwrap();

        // then
        assert_eq!(writer.into(), expected_out);
    }
}
