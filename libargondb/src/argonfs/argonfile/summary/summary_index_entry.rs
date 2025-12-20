use crate::argonfs::argonfile::{
    block::BlockPointer,
    error::ArgonfileWriterError,
    utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
};

pub struct SummaryIndexEntry {
    pub block_ptr: BlockPointer,
    pub key: Box<[u8]>,
}

impl SummaryIndexEntry {
    pub const MIN_SIZE_SERIALIZED: usize = BlockPointer::SERIALIZED_SIZE + 2;

    pub fn serialize(
        writer: &mut impl ArgonfileWrite,
        entry: &SummaryIndexEntry,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        BlockPointer::serialize(&mut writer, &entry.block_ptr)?;

        let key_size = entry.key.len();
        assert!(key_size <= u16::MAX as usize);
        writer.write(&u16::to_le_bytes(key_size as u16))?;

        writer.write(&entry.key)?;

        Ok(writer.size())
    }
}
