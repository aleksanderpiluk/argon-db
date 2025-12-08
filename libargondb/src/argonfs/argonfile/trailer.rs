use crate::argonfs::argonfile::ArgonfileDeserializeError;
use crate::argonfs::argonfile::magic::ARGONFILE_MAGIC;

use super::block_ptr::{ArgonfileBlockPointer, ArgonfileBlockPointerWriter};
use super::{
    error::ArgonfileWriterError,
    utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
};

pub struct Trailer {
    pub sstable_id: u64,
    pub summary_block_ptr: ArgonfileBlockPointer,
    pub stats_block_ptr: ArgonfileBlockPointer,
}

impl Trailer {
    pub const SERIALIZED_SIZE: usize = 36;

    pub fn deserialize(buf: &[u8]) -> Result<Trailer, ArgonfileDeserializeError> {
        // let sstable_id = u64::from_be_bytes(bytes)?;

        todo!()
    }

    pub fn serialize(
        w: &mut impl ArgonfileWrite,
        trailer: &Self,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(w);

        writer.write(&u64::to_le_bytes(trailer.sstable_id))?;
        ArgonfileBlockPointerWriter::write(&mut writer, &trailer.summary_block_ptr)?;
        ArgonfileBlockPointerWriter::write(&mut writer, &trailer.stats_block_ptr)?;
        writer.write(ARGONFILE_MAGIC)?;

        Ok(writer.size())
    }
}
