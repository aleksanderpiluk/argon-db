use super::block::BlockPointer;
use super::parse_utils::ensure_size;
use super::{
    error::ArgonfileWriterError,
    utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
};
use crate::argonfs::argonfile::error::ArgonfileParseResult;
use crate::kv::ObjectId;

pub const ARGONFILE_MAGIC: &'static [u8; 8] = b"ARGNFILE";

#[derive(Debug)]
pub struct Trailer {
    pub sstable_id: ObjectId,
    pub summary_block_ptr: BlockPointer,
    pub stats_block_ptr: BlockPointer,
}

impl Trailer {
    pub const SERIALIZED_SIZE: usize = 40;

    pub fn parse(buf: &[u8]) -> ArgonfileParseResult<Trailer> {
        ensure_size(buf.len(), Self::SERIALIZED_SIZE)?;

        let sstable_id = u64::from_le_bytes(buf[0..8].try_into().unwrap());
        let stats_block_ptr = BlockPointer::parse(&buf[8..20])?;
        let summary_block_ptr = BlockPointer::parse(&buf[20..32])?;

        Ok(Self {
            sstable_id: ObjectId(sstable_id),
            stats_block_ptr,
            summary_block_ptr,
        })
    }

    pub fn serialize(
        w: &mut impl ArgonfileWrite,
        trailer: &Self,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(w);

        writer.write(&u64::to_le_bytes(trailer.sstable_id.0))?;
        BlockPointer::serialize(&mut writer, &trailer.stats_block_ptr)?;
        BlockPointer::serialize(&mut writer, &trailer.summary_block_ptr)?;
        writer.write(ARGONFILE_MAGIC)?;

        Ok(writer.size())
    }
}
