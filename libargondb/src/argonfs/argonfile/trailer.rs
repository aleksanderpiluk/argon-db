use crate::argonfs::argonfile::error::ArgonfileParseResult;

use super::{
    error::ArgonfileWriterError,
    utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
};

pub const ARGONFILE_MAGIC: &'static [u8; 8] = b"ARGNFILE";

pub struct Trailer {
    pub sstable_id: u64,
    pub summary_block_ptr: ArgonfileBlockPointer,
    pub stats_block_ptr: ArgonfileBlockPointer,
}

impl Trailer {
    pub const SERIALIZED_SIZE: usize = 36;

    pub fn parse(buf: &[u8]) -> ArgonfileParseResult<Trailer> {
        let sstable_id = u64::from_be_bytes(bytes)?;
        let stats_block_ptr = todo!();
        let summary_block_ptr = todo!();

        Ok(Self {
            sstable_id,
            stats_block_ptr,
            summary_block_ptr,
        })
    }

    pub fn serialize(
        w: &mut impl ArgonfileWrite,
        trailer: &Self,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(w);

        writer.write(&u64::to_le_bytes(trailer.sstable_id))?;
        ArgonfileBlockPointer::serialize(&mut writer, &trailer.summary_block_ptr)?;
        ArgonfileBlockPointer::serialize(&mut writer, &trailer.stats_block_ptr)?;
        writer.write(ARGONFILE_MAGIC)?;

        Ok(writer.size())
    }
}
