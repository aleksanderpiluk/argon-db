use crate::{
    argonfs::argonfile::{
        error::ArgonfileWriterError,
        utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
    },
    kv::mutation::{KVMutation, MutationType},
};

#[derive(Debug)]
pub struct InRowMutation {
    pub timestamp: u64,
    pub column_id: u16,
    pub mutation_type: MutationType,
    pub value: Box<[u8]>,
}

impl InRowMutation {
    pub fn serialize(
        writer: &mut impl ArgonfileWrite,
        mutation: &dyn KVMutation,
    ) -> Result<usize, ArgonfileWriterError> {
        // TODO: ERROR HANDLING
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        writer.write(&u64::to_le_bytes(mutation.timestamp()))?;
        writer.write(&u16::to_le_bytes(mutation.column_id()))?;
        writer.write(&u8::to_le_bytes(mutation.mutation_type() as u8))?;
        writer.write(&u64::to_le_bytes(mutation.value_size()))?;
        writer.write(mutation.value())?;

        Ok(writer.size())
    }
}
