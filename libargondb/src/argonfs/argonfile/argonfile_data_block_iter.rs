use core::panic;

use bytes::Buf;

use crate::{
    argonfs::argonfile::row::{InRowMutation, Row, RowParser},
    kv::{
        KVSSTableDataBlockIter, KVScanIteratorItem,
        mutation::{KVMutation, StructuredMutation},
    },
};

#[derive(Debug)]
pub struct ArgonfileDataBlockIter<B: Buf> {
    buf: B,
    row: Option<Row>,
    idx: usize,
    is_finished: bool,
}

impl<B: Buf> ArgonfileDataBlockIter<B> {
    pub fn new(buf: B) -> Self {
        Self {
            buf,
            row: None,
            idx: 0,
            is_finished: false,
        }
    }

    fn advance(&mut self) {
        if self.is_finished {
            return;
        }

        if let Some(row) = self.row.as_ref() {
            if self.idx + 1 < row.mutations.len() {
                self.idx += 1;
                return;
            }
        }

        if !self.buf.has_remaining() {
            self.is_finished = true;
        } else {
            let row = RowParser::parse(&mut self.buf).unwrap();
            if row.mutations.len() == 0 {
                panic!("no mutations in row");
            }

            self.row = Some(row);
            self.idx = 0;
        }
    }

    fn get_current(&self) -> Option<(&Row, &InRowMutation)> {
        if self.is_finished {
            None
        } else {
            let row = self.row.as_ref().unwrap();
            let mutation = &row.mutations[self.idx];

            Some((row, mutation))
        }
    }
}

impl<B: Buf> KVSSTableDataBlockIter for ArgonfileDataBlockIter<B> {
    fn next(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        self.advance();
        let Some((row, in_row_mutation)) = self.get_current() else {
            return None;
        };

        let primary_key = row.primary_key.clone();

        let timestamp = in_row_mutation.timestamp;
        let column_id = in_row_mutation.column_id;
        let mutation_type = in_row_mutation.mutation_type;
        let value = in_row_mutation.value.clone();

        Some(Box::new(KVSSTableDataBlockIterItem(
            StructuredMutation::try_from(timestamp, column_id, mutation_type, primary_key, value)
                .unwrap(),
        )))
    }
}

// struct ArgonfileDataBlockIterCurrentRow {
//     row: Row,
//     idx: usize,
// }

// impl ArgonfileDataBlockIterCurrentRow {
//     fn new(row: Row) -> Self {
//         Self { row, idx: 0 }
//     }

//     fn next(&mut self) -> Option<(&Row, &InRowMutation)> {

//     }
// }

struct KVSSTableDataBlockIterItem(StructuredMutation);

impl KVScanIteratorItem for KVSSTableDataBlockIterItem {
    fn mutation(&self) -> &(dyn crate::kv::mutation::KVMutation + Send + Sync) {
        &self.0
    }

    fn primary_key(&self) -> &[u8] {
        self.0.primary_key()
    }
}
