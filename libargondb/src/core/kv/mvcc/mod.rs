use std::sync::atomic::AtomicU64;

pub struct MVCC {
    read_id: AtomicU64,
    write_id: AtomicU64, // TODO:
}
