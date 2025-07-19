mod block_cache;
mod futures;
mod read;

use crate::{foundation::block::BlockTag, subsystem::io::futures::GetBlockResult};

pub struct IOSubsystem;

impl IOSubsystem {
    pub fn get_block(block_tag: BlockTag) -> GetBlockResult {}
}
