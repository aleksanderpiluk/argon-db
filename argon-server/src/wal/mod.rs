mod segment;
mod wal_ctx;
mod wal_manager;
mod wal_state;
mod wal_writer;

pub use wal_ctx::WalCtx;
use wal_manager::WALManager;
use wal_state::WALState;
// use wal_writer::WALWriter;

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use segment::Segment;

    use super::*;

    #[test]
    fn test() {
        // let wal_global_state = Arc::new(WALState {
        //     allocating: Segment::new(100).unwrap(),
        // });

        // let wal_state = wal_global_state.clone();
        // let wal_master = thread::spawn(move || {
        //     WALManager::new(wal_state);
        // });

        // let wal_state = wal_global_state.clone();
        // let wal_client = thread::spawn(|| {
        //     WALWriter::new(wal_state);
        // });

        // wal_master.join().unwrap();
        // wal_client.join().unwrap();
    }
}
