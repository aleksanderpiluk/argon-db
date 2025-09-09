use crossbeam_skiplist::SkipSet;

pub struct Memtable {
    inner: SkipSet<()>,
}
