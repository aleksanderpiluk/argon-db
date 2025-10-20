use crate::kv::{
    scan::{ColumnFilter, PrimaryKeyMarker, RangeScanParams, ScanExecutor},
    table::KVTable,
};

pub struct SelectOp {}

impl SelectOp {
    pub async fn execute(&self, table: &KVTable) {
        let from = PrimaryKeyMarker::Start;
        let to = PrimaryKeyMarker::End;
        let scan_params = RangeScanParams::new(from, to, ColumnFilter::All);

        let table_state = table.load_state();
        ScanExecutor::execute(&table_state, scan_params).unwrap();
    }
}
