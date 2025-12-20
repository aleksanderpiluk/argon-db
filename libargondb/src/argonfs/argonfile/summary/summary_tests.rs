use crate::{
    argonfs::argonfile::{
        BlockPointer,
        block::BlockParser,
        summary::{SummaryBuilder, SummaryIndex, SummaryIndexEntry, SummaryParser},
        utils::ArgonfileOffsetCountingWriteWrapper,
    },
    kv::mutation::{MutationType, StructuredMutation},
};

#[test]
fn test_builder_parser_integration() {
    let entries_data = vec![
        (
            BlockPointer {
                offset: 0,
                on_disk_size: 100,
            },
            StructuredMutation::try_from(
                0,
                1,
                MutationType::Put,
                Box::new([1, 2]),
                Box::new([1, 2]),
            )
            .unwrap(),
        ),
        (
            BlockPointer {
                offset: 100,
                on_disk_size: 150,
            },
            StructuredMutation::try_from(
                12,
                1,
                MutationType::Put,
                Box::new([1, 2]),
                Box::new([1, 2]),
            )
            .unwrap(),
        ),
        (
            BlockPointer {
                offset: 250,
                on_disk_size: 100,
            },
            StructuredMutation::try_from(
                34,
                1,
                MutationType::Put,
                Box::new([1, 2]),
                Box::new([1, 2]),
            )
            .unwrap(),
        ),
        (
            BlockPointer {
                offset: 350,
                on_disk_size: 100,
            },
            StructuredMutation::try_from(
                56,
                1,
                MutationType::Put,
                Box::new([1, 2]),
                Box::new([1, 2]),
            )
            .unwrap(),
        ),
    ];

    let mut summary_builder = SummaryBuilder::new();
    summary_builder
        .next_block_with_min_key(&entries_data[0].1)
        .unwrap();
    summary_builder
        .finish_block_with_ptr(entries_data[0].0)
        .unwrap();

    summary_builder
        .next_block_with_min_key(&entries_data[1].1)
        .unwrap();
    summary_builder
        .finish_block_with_ptr(entries_data[1].0)
        .unwrap();

    summary_builder
        .next_block_with_min_key(&entries_data[2].1)
        .unwrap();
    summary_builder
        .finish_block_with_ptr(entries_data[2].0)
        .unwrap();

    summary_builder
        .next_block_with_min_key(&entries_data[3].1)
        .unwrap();
    summary_builder
        .finish_block_with_ptr(entries_data[3].0)
        .unwrap();

    let mut out_writer = ArgonfileOffsetCountingWriteWrapper::new(Vec::new());
    summary_builder.build(&mut out_writer).unwrap();
    let out_block_buf: Vec<u8> = out_writer.into_inner();

    let block = BlockParser::parse(&out_block_buf).unwrap();
    let summary_index = SummaryParser::parse(&block.data).unwrap();
}
