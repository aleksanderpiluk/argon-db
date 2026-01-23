use std::sync::Arc;

use libargondb::{ArgonFsMemtableFlusherHandle, ConnectorHandle, DbCtx, SSTableCompactorHandle};

pub struct SystemCtx {
    pub db_ctx: Arc<DbCtx>,
    pub memtable_flusher_handle: ArgonFsMemtableFlusherHandle,
    pub sstable_compactor_handle: SSTableCompactorHandle,
    pub connector_handles: Vec<Box<dyn ConnectorHandle>>,
}
