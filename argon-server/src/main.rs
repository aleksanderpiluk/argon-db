use std::thread;

use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    Config,
};
use wal::WalCtx;

mod config;
mod core;
mod db;
mod execution;
mod utils;
mod wal;

fn main() {
    // log::info!("ArgonDB is starting");

    // static wal_ctx: WalCtx = WalCtx::new();

    // thread::spawn(|| {
    //     log::info!("Spawned the WAL master thread");
    // });

    // static db_ctx: DbCtx = DbCtx::new();

    // thread::spawn(|| {
    //     log::info!("Spawned the DB master thread");
    // });

    // let term_now = Arc::new(AtomicBool::new(false));
    // for sig in signal_hook::consts::TERM_SIGNALS {
    //     flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now)).unwrap();
    //     flag::register(*sig, Arc::clone(&term_now)).unwrap();
    // }

    // let core_ctl = CoreCtl::init();

    // while !term_now.load(Ordering::Relaxed) {
    //     std::thread::yield_now();
    // }

    // core_ctl.shutdown();
}
