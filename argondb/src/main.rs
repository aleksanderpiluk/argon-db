mod connectors;
mod errors;
mod exit;
mod init;
mod ops;
mod shutdown;
mod signals_handler;
mod supervisor;
mod system_tables;

use libargondb::{ArgonFsMemtableFlusher, SSTableCompactor};

use crate::{
    connectors::grpc::init_connector_grpc,
    errors::{OkOrAbort, OrCriticalError},
    init::run_init_thread,
    shutdown::run_shutdown_thread,
    signals_handler::handle_signals,
    supervisor::{SystemCtx, run_supervisor_thread},
};

fn main() {
    println!("argondb is starting");
    let db_ctx = run_init_thread().ok_or_abort();

    let memtable_flusher_handle = ArgonFsMemtableFlusher::new(db_ctx.clone());

    let sstable_compactor_handle = SSTableCompactor::new(db_ctx.clone());

    let connector_handle = init_connector_grpc(db_ctx.clone())
        .ok_or_critical_err()
        .ok_or_abort();

    let system_ctx = SystemCtx {
        db_ctx: db_ctx.clone(),
        memtable_flusher_handle,
        sstable_compactor_handle,
        connector_handles: vec![connector_handle],
    };

    run_supervisor_thread();

    println!("database is running, watching for signals");
    handle_signals().unwrap();

    run_shutdown_thread(system_ctx).ok_or_abort();
}
