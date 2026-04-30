use std::io::Cursor;

use super::super::parse_utils::ensure_min_size;
use super::Block;
use super::block_header::BlockHeader;
use crate::argonfs::argonfile::block::checksum::{
    ChecksumAlgo, ChecksumAlgoResolver, ChecksumType,
};
use crate::argonfs::argonfile::block::compression::{
    CompressionAlgo, CompressionAlgoResolver, CompressionType,
};
use crate::argonfs::argonfile::error::ArgonfileParseResult;
pub struct BlockParser {}

impl BlockParser {
    pub fn parse(buf: &[u8]) -> ArgonfileParseResult<Block> {
        let block_header = BlockHeader::parse(&buf[0..BlockHeader::SIZE_SERIALIZED])?;

        let buf_compressed_size = block_header.data_compressed_size as usize;
        let buf_checksum_size = block_header.checksum_size as usize;

        let buf = &buf[BlockHeader::SIZE_SERIALIZED..];
        ensure_min_size(buf.len(), buf_compressed_size + buf_checksum_size)?;

        let buf_compressed = &buf[0..buf_compressed_size];

        let buf = &buf[buf_compressed_size..];
        let buf_checksum = &buf[0..buf_checksum_size];

        let compression_type = block_header.compression_type;
        let compression_algo = CompressionAlgoResolver::for_compression_type(compression_type);

        let decompressed_size = block_header.data_uncompressed_size as usize;
        let buf_decompressed = vec![0u8; decompressed_size].into_boxed_slice();

        let mut buf_decompressed_writer = Cursor::new(buf_decompressed);
        compression_algo.decompress(
            buf_compressed,
            &mut buf_decompressed_writer,
            decompressed_size,
        )?;
        let buf_decompressed = buf_decompressed_writer.into_inner();

        let checksum_type = block_header.checksum_type;
        let checksum_algo = ChecksumAlgoResolver::for_checksum_type(checksum_type);

        checksum_algo.verify_checksum(&buf_decompressed, buf_checksum)?;

        Ok(Block {
            data: buf_decompressed,
            checksum_type,
            compression_type,
        })
    }
}
