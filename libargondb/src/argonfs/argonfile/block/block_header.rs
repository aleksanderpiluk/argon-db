use super::super::parse_utils::ensure_size;
use super::block_identifier::BlockIdentifier;
use super::checksum::ChecksumType;
use super::compression::CompressionType;
use crate::argonfs::argonfile::{
    error::{ArgonfileParseResult, ArgonfileWriterError},
    utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
};

pub struct BlockHeader {
    pub block_identifier: BlockIdentifier,
    pub data_compressed_size: u32,
    pub data_uncompressed_size: u32,
    pub compression_type: CompressionType,
    pub checksum_type: ChecksumType,
    pub checksum_size: u32,
}

impl BlockHeader {
    pub const SIZE_SERIALIZED: usize = 24;

    pub fn parse(buf: &[u8]) -> ArgonfileParseResult<Self> {
        ensure_size(buf.len(), BlockHeader::SIZE_SERIALIZED)?;

        let block_identifier: [u8; 8] = buf[0..8].try_into().unwrap();
        let data_compressed_size = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        let data_uncompressed_size = u32::from_le_bytes(buf[12..16].try_into().unwrap());
        let checksum_type = u8::from_le_bytes(buf[16..17].try_into().unwrap());
        let compression_type = u8::from_le_bytes(buf[17..18].try_into().unwrap());
        let checksum_size = u32::from_le_bytes(buf[20..24].try_into().unwrap());

        let compression_type = CompressionType::try_from(compression_type)?;
        let checksum_type = ChecksumType::try_from(checksum_type)?;

        Ok(BlockHeader {
            block_identifier,
            data_compressed_size,
            data_uncompressed_size,
            compression_type,
            checksum_type,
            checksum_size,
        })
    }

    pub fn serialize(
        writer: &mut impl ArgonfileWrite,
        block_identifier: &BlockIdentifier,
        data_compressed_size: u32,
        data_uncompressed_size: u32,
        checksum_type: ChecksumType,
        checksum_size: u32,
        compression_type: CompressionType,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        writer.write(block_identifier)?;
        writer.write(&u32::to_le_bytes(data_compressed_size))?;
        writer.write(&u32::to_le_bytes(data_uncompressed_size))?;
        writer.write(&u8::to_le_bytes(checksum_type.into()))?;
        writer.write(&u8::to_le_bytes(compression_type.into()))?;
        writer.write(&[0u8; 2])?; // Reserved space
        writer.write(&u32::to_le_bytes(checksum_size))?;

        Ok(writer.size())
    }
}
