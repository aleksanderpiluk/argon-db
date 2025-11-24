mod argonfile_format_reader;
mod block;
mod block_identifier;
mod block_ptr;
mod builder;
mod checksum;
mod compression;
mod config;
mod error;
mod magic;
mod row;
mod stats;
mod summary;
mod trailer;
mod utils;

<<<<<<< HEAD:libargondb/src/argonfile/mod.rs
// pub struct ArgonfileReader {
//     compression: ArgonfileCompressionType,
// }

pub use reader::ArgonfileReader;
=======
pub use argonfile_format_reader::ArgonfileFormatReader;
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/argonfile/mod.rs

pub enum ArgonfileCompressionType {
    None,
}

use bytemuck::from_bytes;

fn summary_block_lookup() {
    let data: &[u8] = &[0, 1, 2, 3, 4]; // TODO:

    let mut iter = SummaryBlockIterator::new(data);
}

fn index_block_lookup() {
    let data: &[u8] = &[0, 1, 2, 3, 4]; // TODO:

    let mut iter = IndexBlockIterator::new(data);
}

struct SummaryBlockIterator<'a> {
    data: &'a [u8],
    ptr: usize,
}

impl<'a> SummaryBlockIterator<'a> {
    fn new(data: &'a [u8]) -> Self {
        // TODO: CHANGE PTR FROM 0
        Self { data, ptr: 0 }
    }
}

impl<'a> Iterator for SummaryBlockIterator<'a> {
    type Item = SummaryBlockEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.data;
        let ptr = self.ptr;

        if self.ptr >= data.len() {
            return None;
        }

        let block_ptr = *from_bytes::<u64>(&data[ptr..ptr + 64]);
        let disk_size = *from_bytes::<u32>(&data[ptr + 64..ptr + 96]);
        let key_size = *from_bytes::<u16>(&data[ptr + 96..ptr + 112]);
        let key = &data[ptr + 112..ptr + 112 + key_size as usize];

        self.ptr = ptr + 112 + key_size as usize;
        Some(SummaryBlockEntry {
            block_ptr,
            disk_size,
            key,
        })
    }
}

struct SummaryBlockEntry<'a> {
    block_ptr: u64,
    disk_size: u32,
    key: &'a [u8],
}

struct IndexBlockIterator<'a> {
    data: &'a [u8],
    ptr: usize,
}

impl<'a> IndexBlockIterator<'a> {
    fn new(data: &'a [u8]) -> Self {
        // TODO: CHANGE PTR FROM 0
        Self { data, ptr: 0 }
    }
}

impl<'a> Iterator for IndexBlockIterator<'a> {
    type Item = IndexBlockEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.data;
        let ptr = self.ptr;

        if self.ptr >= data.len() {
            return None;
        }

        let block_ptr = *from_bytes::<u64>(&data[ptr..ptr + 64]);
        let disk_size = *from_bytes::<u32>(&data[ptr + 64..ptr + 96]);
        let row_ptr = *from_bytes::<u32>(&data[ptr + 96..ptr + 128]);
        let key_size = *from_bytes::<u16>(&data[ptr + 128..ptr + 144]);
        let key = &data[ptr + 144..ptr + 144 + key_size as usize];

        self.ptr = ptr + 144 + key_size as usize;
        Some(IndexBlockEntry {
            block_ptr,
            disk_size,
            row_ptr,
            key,
        })
    }
}

struct IndexBlockEntry<'a> {
    block_ptr: u64,
    disk_size: u32,
    row_ptr: u32,
    key: &'a [u8],
}
