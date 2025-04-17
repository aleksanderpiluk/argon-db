use anyhow::{anyhow, Result};
use bloomfilter::Bloom;
use libargondb::PartitionKey;

use crate::block::{Block, BlockIdentifier};

#[derive(Debug)]
pub struct BloomBuilder {
    bloom: Bloom<PartitionKey>,
}

impl BloomBuilder {
    const BLOOM_FILTER_FP_CHANCE: f64 = 0.05;

    pub fn new(items_count: usize) -> Result<Self> {
        let bloom = Bloom::new_for_fp_rate(items_count, Self::BLOOM_FILTER_FP_CHANCE)
            .map_err(|err| anyhow!(err))?;

        Ok(Self { bloom })
    }

    pub fn add_partition_key(&mut self, key: &PartitionKey) {
        self.bloom.set(key);
    }

    pub fn close(self) -> Result<Block> {
        Block::new(
            BlockIdentifier::BLOOM_BLOCK,
            self.bloom.into_bytes().into_boxed_slice(),
        )
    }
}
