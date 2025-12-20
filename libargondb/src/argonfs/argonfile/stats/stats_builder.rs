use std::sync::Arc;

use bloomfilter::Bloom;

use super::super::block::BlockPointer;
use crate::{
    argonfs::argonfile::{
        block::{
            BLOCK_IDENTIFIER_STATS, BlockBuilder,
            checksum::{ChecksumAlgoResolver, ChecksumType},
            compression::{CompressionAlgoResolver, CompressionType},
        },
        error::ArgonfileBuilderError,
        stats::Stats,
        utils::{ArgonfileOffsetCountingWriteWrapper, ArgonfileWrite},
    },
    kv::{memtable::Memtable, mutation::KVMutation},
};

pub struct StatsBuilder {
    memtable: Arc<Memtable>,
    bloom: Bloom<[u8]>,
    min_key: Option<Box<[u8]>>,
    max_key: Option<Box<[u8]>>,
}

impl StatsBuilder {
    const BLOOM_FILTER_FP_CHANCE: f64 = 0.05;

    pub fn new(memtable: Arc<Memtable>) -> Result<Self, ArgonfileBuilderError> {
        let pre_stats = memtable
            .get_flush_prestats()
            .map_err(|e| ArgonfileBuilderError::from_source(e))?;
        let mutations_count = pre_stats.mutations_count;

        let bloom =
            Bloom::new_for_fp_rate(mutations_count, Self::BLOOM_FILTER_FP_CHANCE).map_err(|e| {
                ArgonfileBuilderError::from_msg(format!("Bloom construction error: {}", e))
            })?;

        Ok(Self {
            memtable,
            bloom,
            min_key: None,
            max_key: None,
        })
    }

    pub fn add_mutation(&mut self, mutation: &dyn KVMutation) {
        let key = mutation.primary_key();

        if self.min_key.is_none() {
            self.min_key = Some(key.to_owned().into_boxed_slice());
        }
        self.max_key = Some(key.to_owned().into_boxed_slice());

        self.bloom.set(mutation.primary_key());
    }

    pub fn build(
        self,
        writer: &mut impl ArgonfileWrite,
    ) -> Result<BlockPointer, ArgonfileBuilderError> {
        let mut block_builder = BlockBuilder::new(0);

        let min_row_key = self.min_key.ok_or(ArgonfileBuilderError::from_msg(
            "no min row key in stat builder",
        ))?;
        let max_row_key = self.max_key.ok_or(ArgonfileBuilderError::from_msg(
            "no max row key in stat builder",
        ))?;

        let stats = Stats {
            bloom_filter: self.bloom.to_bytes().into_boxed_slice(),
            min_row_key,
            max_row_key,
        };

        let buf = Vec::<u8>::new();
        let mut stats_writer = ArgonfileOffsetCountingWriteWrapper::new(buf);

        Stats::serialize(&mut stats_writer, &stats)?;

        let buf = stats_writer.into_inner();
        block_builder
            .write(&buf)
            .map_err(|e| ArgonfileBuilderError::from_msg("ArgonfileBlockWriterError"))?;

        let checksum_algo = ChecksumAlgoResolver::for_checksum_type(ChecksumType::CRC32);
        let compression_algo =
            CompressionAlgoResolver::for_compression_type(CompressionType::Uncompressed);

        let ptr = block_builder.build(
            writer,
            BLOCK_IDENTIFIER_STATS,
            &checksum_algo,
            &compression_algo,
        )?;

        Ok(ptr)
    }
}
