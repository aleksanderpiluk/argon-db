use std::io::{self};

use crate::argonfs::argonfile::block::{checksum::ChecksumType, compression::CompressionType};

pub struct Block {
    pub data: Box<[u8]>,
    pub checksum_type: ChecksumType,
    pub compression_type: CompressionType,
}

#[derive(Debug)]
pub enum ArgonfileBlockWriterError {
    IOError(io::Error),
    PartialWrite(usize),
}

impl From<io::Error> for ArgonfileBlockWriterError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}
