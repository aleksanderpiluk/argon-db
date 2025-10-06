use std::sync::Arc;

#[derive(Clone)]
pub struct TableStateSnapshot {
    pub table_name: String,
    schema: (),
    // current_memtable: Arc<Memtable>,
}

impl TableStateSnapshot {
    fn table_name(&self) -> &String {
        &self.table_name
    }

    // fn current_memtable(&self) -> &Arc<Memtable> {
    //     &self.current_memtable
    // }
}
