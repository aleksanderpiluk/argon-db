use std::io::Write;

use anyhow::Result;

use crate::{
    block::{Block, BlockIdentifier},
    shared::{PositionedWriter, Writer},
};

use super::{summary_entry_writer::SummaryEntryWriter, SummaryEntry};

pub struct SummaryBuilder;

impl SummaryBuilder {
    pub fn block_from_entries(entries: Vec<SummaryEntry>) -> Result<Block> {
        let mut buf = PositionedWriter::new(Vec::<u8>::new());

        buf.write(&u64::to_be_bytes(entries.len() as u64))?;
        for entry in entries {
            SummaryEntryWriter::try_write(&mut buf, &entry)?;
        }

        Block::new(
            BlockIdentifier::SUMMARY_BLOCK,
            buf.into().into_boxed_slice(),
        )
    }
}
