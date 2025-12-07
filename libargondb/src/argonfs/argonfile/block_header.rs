use std::array::TryFromSliceError;

use crate::argonfs::argonfile::{
    block_identifier::BlockIdentifier,
    checksum::{ChecksumType, ChecksumTypeParseError},
    compression::{CompressionType, CompressionTypeParseError},
    error::ArgonfileWriterError,
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
}

pub struct BlockHeaderReader;

impl BlockHeaderReader {
    pub fn read(buf: &[u8]) -> Result<BlockHeader, BlockHeaderReaderError> {
        let header_bytes = <[u8; BlockHeader::SIZE_SERIALIZED]>::try_from(buf)?;

        let block_identifier: [u8; 8] = <[u8; 8]>::try_from(&buf[0..8])?;
        let data_compressed_size = u32::from_le_bytes(<[u8; 4]>::try_from(&buf[8..12])?);
        let data_uncompressed_size = u32::from_le_bytes(<[u8; 4]>::try_from(&buf[12..16])?);
        let checksum_type = u8::from_le_bytes(<[u8; 1]>::try_from(&buf[16..17])?);
        let compression_type = u8::from_le_bytes(<[u8; 1]>::try_from(&buf[17..18])?);
        let checksum_size = u32::from_le_bytes(<[u8; 4]>::try_from(&buf[20..24])?);

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
}

pub struct BlockHeaderReaderError;

impl From<ChecksumTypeParseError> for BlockHeaderReaderError {
    fn from(value: ChecksumTypeParseError) -> Self {
        todo!()
    }
}

impl From<CompressionTypeParseError> for BlockHeaderReaderError {
    fn from(value: CompressionTypeParseError) -> Self {
        todo!()
    }
}

impl From<TryFromSliceError> for BlockHeaderReaderError {
    fn from(value: TryFromSliceError) -> Self {
        todo!()
    }
}

pub struct BlockHeaderWriter;

impl BlockHeaderWriter {
    pub fn write(
        writer: &mut impl ArgonfileWrite,
        block_identifier: &BlockIdentifier,
        data_compressed_size: u32,
        data_uncompressed_size: u32,
        checksum_type: ChecksumType,
        checksum_size: u32,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        writer.write(block_identifier)?;
        writer.write(&u32::to_le_bytes(data_compressed_size))?;
        writer.write(&u32::to_le_bytes(data_uncompressed_size))?;
        writer.write(&u8::to_le_bytes(checksum_type.into()))?;
        writer.write(&[0u8; 3])?; // Reserved space
        writer.write(&u32::to_le_bytes(checksum_size))?;

        Ok(writer.size())
    }
}
