use bytes::Buf;

use crate::{
    argonfs::argonfile::{
        error::{ArgonfileParseError, ArgonfileParseResult},
        row::in_row_mutation::InRowMutation,
    },
    kv::mutation::{KVMutation, MutationType, StructuredMutation},
};

pub struct InRowMutationParser;

impl InRowMutationParser {
    pub fn parse(buf: &mut impl Buf) -> ArgonfileParseResult<InRowMutation> {
        let timestamp = buf.get_u64_le();
        let column_id = buf.get_u16_le();

        let mutation_type = buf.get_u8();
        let mutation_type =
            MutationType::try_from(mutation_type).map_err(|_| ArgonfileParseError)?;

        let value_size = buf.get_u64_le();

        let mut value = vec![0u8; value_size as usize].into_boxed_slice();
        buf.copy_to_slice(&mut value);

        Ok(InRowMutation {
            timestamp,
            column_id,
            mutation_type,
            value,
        })
    }
}
