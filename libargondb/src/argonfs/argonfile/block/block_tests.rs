use crate::argonfs::argonfile::{
    block::{
        BLOCK_IDENTIFIER_DATA, BlockBuilder, BlockParser,
        checksum::{ChecksumAlgoResolver, ChecksumType},
        compression::{CompressionAlgoResolver, CompressionType},
    },
    utils::ArgonfileOffsetCountingWriteWrapper,
};

#[test]
fn test_builder_parser_integration() {
    let mut block_builder = BlockBuilder::new(1024);
    let checksum_algo = ChecksumAlgoResolver::for_checksum_type(ChecksumType::CRC32);
    let compression_algo = CompressionAlgoResolver::for_default_compression_type();
    let data_buf: Vec<u8> = vec![
        15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
        12, 13, 14, 15,
    ];

    block_builder.write(&data_buf).unwrap();

    let mut out_writer = ArgonfileOffsetCountingWriteWrapper::new(Vec::new());
    block_builder
        .build(
            &mut out_writer,
            BLOCK_IDENTIFIER_DATA,
            &checksum_algo,
            &compression_algo,
        )
        .unwrap();
    let out_buf: Vec<u8> = out_writer.into_inner();

    let block = BlockParser::parse(&out_buf).unwrap();
}
