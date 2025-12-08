use std::io::Write;

use bloomfilter::Bloom;
use bytes::Buf;

use super::block_ptr::ArgonfileBlockPointer;
use crate::{
    argonfs::argonfile::{ArgonfileReaderError, utils::ArgonfileWrite},
    kv::{KVSSTableStats, mutation::KVMutation},
};

pub struct Stats {
    pub min_row: Box<[u8]>,
    pub max_row: Box<[u8]>,
}

impl Stats {
    pub fn deserialize(buf: impl Buf) -> Result<Stats, ArgonfileReaderError> {
        todo!()
    }
}

impl Into<KVSSTableStats> for Stats {
    fn into(self) -> KVSSTableStats {
        KVSSTableStats {
            min_row: self.min_row,
            max_row: self.max_row,
        }
    }
}

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
