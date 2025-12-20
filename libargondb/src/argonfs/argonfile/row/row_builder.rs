use crate::{
    argonfs::argonfile::{
        error::ArgonfileBuilderError,
        row::{in_row_mutation::InRowMutation, row::Row},
        utils::{ArgonfileOffsetCountingWriteWrapper, ArgonfileWrite},
    },
    ensure,
    kv::mutation::KVMutation,
};

pub struct RowBuilder {
    primary_key: Box<[u8]>,
    buffer: ArgonfileOffsetCountingWriteWrapper<Vec<u8>>,
}

impl RowBuilder {
    pub fn new(primary_key: Box<[u8]>) -> Self {
        Self {
            primary_key,
            buffer: ArgonfileOffsetCountingWriteWrapper::new(vec![]),
        }
    }

    pub fn belongs_to_row<T: KVMutation + ?Sized>(&self, mutation: &T) -> bool {
        let row_pk = &self.primary_key[..];
        let mutation_pk = mutation.primary_key();

        return row_pk.eq(mutation_pk);
    }

    pub fn add_mutation(
        &mut self,
        mutation: &dyn KVMutation,
    ) -> Result<usize, ArgonfileBuilderError> {
        ensure!(
            self.belongs_to_row(mutation),
            ArgonfileBuilderError::from_msg("mutation does not belong to builded row")
        );

        Ok(InRowMutation::serialize(&mut self.buffer, mutation)?)
    }

    pub fn end_row(self, w: &mut impl ArgonfileWrite) -> Result<usize, ArgonfileBuilderError> {
        let buffer = self.buffer.into_inner();
        Row::serialize(w, &self.primary_key, &buffer).map_err(|e| e.into())
    }
}
