use bytes::Buf;

use crate::argonfs::argonfile::{
    error::ArgonfileParseResult,
    row::{in_row_mutation_parser::InRowMutationParser, row::Row},
};

pub struct RowParser;

impl RowParser {
    pub fn parse(buf: &mut impl Buf) -> ArgonfileParseResult<Row> {
        let primary_key_size = buf.get_u16_le();
        let mutations_size = buf.get_u32_le();

        let mut primary_key = vec![0u8; primary_key_size as usize].into_boxed_slice();
        buf.copy_to_slice(&mut primary_key);

        let mut mutations = vec![0u8; mutations_size as usize].into_boxed_slice();
        buf.copy_to_slice(&mut mutations);

        let mut ptr = mutations.as_ref();

        let mut in_row_mutations = vec![];
        while ptr.has_remaining() {
            let in_row_mutation = InRowMutationParser::parse(&mut ptr)?;

            in_row_mutations.push(in_row_mutation);
        }

        Ok(Row {
            primary_key,
            mutations: in_row_mutations,
        })
    }
}
