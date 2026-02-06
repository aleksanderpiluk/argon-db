pub struct State {
    tables: Vec<super::Table>,
    memtable_flush_queue: (),
}
