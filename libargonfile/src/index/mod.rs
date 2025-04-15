use crate::pointer::Pointer;

mod index_builder;
mod index_entry_reader;
mod index_entry_writer;

pub use index_builder::IndexBuilder;
pub use index_entry_writer::IndexEntryWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    block_ptr: Pointer,
    partition_offset: u32,
    key: Box<[u8]>,
}

impl IndexEntry {
    pub fn into_key(mut self) -> Box<[u8]> {
        self.key
    }
}

pub struct SummaryEntry {
    block_ptr: Pointer,
    key: Box<[u8]>,
}

impl SummaryEntry {
    pub fn new(block_ptr: Pointer, key: Box<[u8]>) -> Self {
        Self { block_ptr, key }
    }
}
