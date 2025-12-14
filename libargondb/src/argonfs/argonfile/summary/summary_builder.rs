use crate::{
    argonfs::argonfile::{ArgonfileBlockPointer, utils::ArgonfileWrite},
    kv::mutation::KVMutation,
};

pub struct SummaryBuilder {
    block_builder: (),
}

impl SummaryBuilder {
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
