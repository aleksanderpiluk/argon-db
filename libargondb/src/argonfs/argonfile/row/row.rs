use crate::{
    argonfs::argonfile::{
        error::ArgonfileWriterError,
        row::in_row_mutation::InRowMutation,
        utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
    },
    kv::primary_key::KVPrimaryKeyUtils,
};

#[derive(Debug)]
pub struct Row {
    pub primary_key: Box<[u8]>,
    pub mutations: Vec<InRowMutation>,
}

impl Row {
    pub fn serialize(
        w: &mut impl ArgonfileWrite,
        primary_key: &[u8],
        mutations: &[u8],
    ) -> Result<usize, ArgonfileWriterError> {
        let primary_key_size = KVPrimaryKeyUtils::size(primary_key);
        let mutations_size = mutations.len();
        assert!(mutations_size <= u32::MAX as usize);

        let mut writer = ArgonfileSizeCountingWriter::new(w);

        writer.write(&u16::to_le_bytes(primary_key_size))?;
        writer.write(&u32::to_le_bytes(mutations_size as u32))?;
        writer.write(primary_key)?;
        writer.write(mutations)?;

        Ok(writer.size())
    }
}
