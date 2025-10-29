use crate::{argonfile::utils::ArgonfileWrite, kv::mutation::KVMutation};

use super::block_ptr::ArgonfileBlockPointer;
use std::io::Write;

pub struct ArgonfileSummaryBuilder {
    block_builder: (),
}

impl ArgonfileSummaryBuilder {
    pub fn new() -> Self {
        todo!()
    }

    pub fn next_block_with_min_key(&mut self, mutation: &impl KVMutation) {
        todo!()
    }

    pub fn finish_block_with_ptr(&mut self, block_ptr: ArgonfileBlockPointer) {
        todo!()
    }

    pub fn build(self, writer: &mut impl ArgonfileWrite) -> ArgonfileBlockPointer {
        todo!()
    }
}
