use std::io::Write;

use bloomfilter::Bloom;

use super::block_ptr::ArgonfileBlockPointer;
use crate::{argonfs::argonfile::utils::ArgonfileWrite, kv::mutation::KVMutation};

pub struct ArgonfileStatsBuilder {
    bloom: Bloom<[u8]>,
}

impl ArgonfileStatsBuilder {
    const BLOOM_FILTER_FP_CHANCE: f64 = 0.05;

    pub fn new(mutations_count: usize) -> Self {
        Self {
            // TODO: Better error handling
            bloom: Bloom::new_for_fp_rate(mutations_count, Self::BLOOM_FILTER_FP_CHANCE).unwrap(),
        }
    }

    pub fn add_mutation(&mut self, mutation: &impl KVMutation) {
        self.bloom.set(mutation.primary_key());
    }

    pub fn build(self, writer: &mut impl ArgonfileWrite) -> ArgonfileBlockPointer {
        todo!()
    }
}
