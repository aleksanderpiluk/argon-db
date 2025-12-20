use super::super::parse_utils::ensure_size;
use crate::argonfs::argonfile::{
    error::{ArgonfileParseResult, ArgonfileWriterError},
    utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlockPointer {
    pub offset: u64,
    pub on_disk_size: u32,
}

impl BlockPointer {
    pub const SERIALIZED_SIZE: usize = 12;

    pub fn new(offset: u64, on_disk_size: u32) -> Self {
        Self {
            offset,
            on_disk_size,
        }
    }

    pub fn parse(buf: &[u8]) -> ArgonfileParseResult<Self> {
        ensure_size(buf.len(), Self::SERIALIZED_SIZE)?;

        let offset = u64::from_le_bytes(buf[0..8].try_into().unwrap());
        let on_disk_size = u32::from_le_bytes(buf[8..12].try_into().unwrap());

        Ok(Self {
            offset,
            on_disk_size,
        })
    }

    pub fn serialize(
        w: &mut impl ArgonfileWrite,
        block_ptr: &Self,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(w);

        writer.write(&u64::to_le_bytes(block_ptr.offset))?;
        writer.write(&u32::to_le_bytes(block_ptr.on_disk_size))?;

        Ok(writer.size())
    }
}
