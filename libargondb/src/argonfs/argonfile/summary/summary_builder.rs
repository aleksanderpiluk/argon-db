use std::{mem::replace, usize};

use crate::{
    argonfs::argonfile::{
        block::{
            BLOCK_IDENTIFIER_SUMMARY, BlockBuilder, BlockPointer,
            checksum::{ChecksumAlgoResolver, ChecksumType},
            compression::{CompressionAlgoResolver, CompressionType},
        },
        error::ArgonfileBuilderError,
        summary::{SummaryIndex, SummaryIndexEntry},
        utils::{ArgonfileOffsetCountingWriteWrapper, ArgonfileSizeCountingWriter, ArgonfileWrite},
    },
    kv::mutation::KVMutation,
};

pub struct SummaryBuilder {
    entries: Vec<SummaryIndexEntry>,
    current_block: Option<Box<[u8]>>,
}

impl SummaryBuilder {
    pub fn new() -> Self {
        Self {
            entries: vec![],
            current_block: None,
        }
    }

    pub fn next_block_with_min_key(
        &mut self,
        mutation: &dyn KVMutation,
    ) -> Result<(), ArgonfileBuilderError> {
        if self.current_block.is_some() {
            return Err(ArgonfileBuilderError::from_msg(
                "cannot create next block - current block not flushed",
            ));
        }

        let key = Box::from(mutation.primary_key());
        self.current_block = Some(key);

        Ok(())
    }

    pub fn finish_block_with_ptr(
        &mut self,
        block_ptr: BlockPointer,
    ) -> Result<(), ArgonfileBuilderError> {
        let Some(current_block) = replace(&mut self.current_block, None) else {
            return Err(ArgonfileBuilderError::from_msg(
                "cannot flush current block - block not exists",
            ));
        };

        self.entries.push(SummaryIndexEntry {
            block_ptr,
            key: current_block,
        });

        Ok(())
    }

    pub fn build(
        self,
        writer: &mut impl ArgonfileWrite,
    ) -> Result<BlockPointer, ArgonfileBuilderError> {
        let mut block_builder = BlockBuilder::new(0);

        let summary_index = SummaryIndex {
            entries: self.entries,
        };

        let buf = Vec::<u8>::new();
        let mut index_writer = ArgonfileOffsetCountingWriteWrapper::new(buf);

        SummaryIndex::serialize(&mut index_writer, &summary_index)?;

        let buf = index_writer.into_inner();
        block_builder
            .write(&buf)
            .map_err(|e| ArgonfileBuilderError::from_msg("ArgonfileBlockWriterError"))?;

        let checksum_algo = ChecksumAlgoResolver::for_checksum_type(ChecksumType::CRC32);
        let compression_algo = CompressionAlgoResolver::for_default_compression_type();

        let ptr = block_builder.build(
            writer,
            BLOCK_IDENTIFIER_SUMMARY,
            &checksum_algo,
            &compression_algo,
        )?;

        Ok(ptr)
    }
}
