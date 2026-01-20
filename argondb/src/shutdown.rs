use async_io::block_on;
use libargondb::ConnectorHandle;
use libargondb::DbCtx;
use std::thread;

use crate::errors::CriticalError;
use crate::{
    errors::{CriticalResult, OrCriticalError},
    supervisor::SystemCtx,
};

pub fn run_shutdown_thread(system_ctx: SystemCtx) -> CriticalResult<()> {
    println!("spawning shutdown thread");

    let thread_handle = thread::spawn(|| shutdown_thread(system_ctx));

    let result = thread_handle
        .join()
        .map_err(|_| CriticalError::from_msg("failed to join on shutdown thread"))?;

    result.map_err(|err| {
        CriticalError::from_msg_and_source("critical error while running shutdown thread", err)
    })?;

    println!("shutdown thread joined successfully");

    Ok(())
}

fn shutdown_thread(system_ctx: SystemCtx) -> CriticalResult<()> {
    println!("shutdown thread - running shutdown procedure");

    close_connectors(system_ctx.connector_handles)?;

    close_kv_instance_and_tables(&system_ctx.db_ctx)?;
    system_ctx.sstable_compactor_handle.close();
    system_ctx.memtable_flusher_handle.close();

    persist_instance_state_snapshot(&system_ctx.db_ctx)?;

    println!("shutdown thread - shutdown procedure finished");
    Ok(())
}

fn persist_instance_state_snapshot(db_ctx: &DbCtx) -> CriticalResult<()> {
    let snapshot = db_ctx.kv_instance.state_snapshot();

    println!(
        "shutdown thread - persisting instance state snapshot {:?}",
        snapshot
    );
    smol::block_on(db_ctx.persistence.save_instance_snapshot(snapshot)).ok_or_critical_err()?;

    Ok(())
}

pub fn close_connectors(connector_handles: Vec<Box<dyn ConnectorHandle>>) -> CriticalResult<()> {
    for handle in connector_handles {
        block_on(handle.close());
    }

    Ok(())
}

pub fn close_kv_instance_and_tables(db_ctx: &DbCtx) -> CriticalResult<()> {
    let tables = db_ctx.catalog.list_tables();

    for table in tables {
        table.close();
    }

    db_ctx.kv_instance.close();

    Ok(())
}
