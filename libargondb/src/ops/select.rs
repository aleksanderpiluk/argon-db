use crate::kv::{KVColumnFilter, KVRangeScan, KVRowIter, KVTable, primary_key::KVPrimaryKeyMarker};

pub struct SelectOp {}

impl SelectOp {
    pub async fn execute(&self, table: &KVTable) {
        let from = KVPrimaryKeyMarker::Start;
        let to = KVPrimaryKeyMarker::End;
        let scan_params =
            KVRangeScan::new(table.table_schema.clone(), from, to, KVColumnFilter::All);

        let mut scan = table.scan(scan_params).await.unwrap();

        while let Some(row) = scan.next_row().await.unwrap() {
            println!("{:?}", row);
        }
    }
}

// struct ScanResult(Box<[u8]>);

// impl ScanResult {
//     pub fn iter(&self) -> ScanResultIterator<'_> {
//         ScanResultIterator {
//             data: &self,
//             ptr: 0,
//         }
//     }
// }

// struct ScanResultIterator<'a> {
//     data: &'a ScanResult,
//     ptr: usize,
// }

// impl<'a> Iterator for ScanResultIterator<'a> {
//     type Item = InMemoryMutationView<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.ptr < self.data.0.len() {
//             let item = InMemoryMutationView::try_from(&self.data.0[self.ptr..]).unwrap();
//             self.ptr += item.len();

//             Some(item)
//         } else {
//             None
//         }
//     }
// }

// pub struct ScanResultBuilder(Vec<u8>);

// impl ScanResultBuilder {
//     pub fn new() -> Self {
//         Self(Vec::new())
//     }

//     pub fn write(&mut self, mutation: impl Mutation) {
//         // TODO: Assert mutations are put in order
//         InMemoryMutationView::write(&mut self.0, mutation);
//     }

//     pub fn build(self) -> ScanResult {
//         ScanResult(self.0.into_boxed_slice())
//     }
// }
