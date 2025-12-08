use bytes::Buf;

use super::utils::ArgonfileWrite;
use crate::{
    argonfs::argonfile::ArgonfileReaderError,
    kv::{KVSSTableSummaryIndex, mutation::KVMutation},
};

use super::block_ptr::ArgonfileBlockPointer;
use std::io::Write;

pub struct Summary {}

impl Summary {
    pub fn deserialize(buf: impl Buf) -> Result<Summary, ArgonfileReaderError> {
        todo!()
    }
}

impl Into<KVSSTableSummaryIndex> for Summary {
    fn into(self) -> KVSSTableSummaryIndex {
        KVSSTableSummaryIndex { entries: todo!() }
    }
}

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
