mod api_grpc;
mod block;
mod block_cache;
mod cell;
mod core;
mod execution;
mod lock;
mod memstore;
mod store_file;
mod table;
mod wal;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use signal_hook::flag;

use self::core::CoreCtl;

fn main() {
    let term_now = Arc::new(AtomicBool::new(false));
    for sig in signal_hook::consts::TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now)).unwrap();
        flag::register(*sig, Arc::clone(&term_now)).unwrap();
    }

    let core_ctl = CoreCtl::init();

    while !term_now.load(Ordering::Relaxed) {
        std::thread::yield_now();
    }

    core_ctl.shutdown();
}
