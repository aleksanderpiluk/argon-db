use std::sync::Arc;

use libargondb::{ArgonFsMemtableFlusherHandle, ConnectorHandle, DbCtx};

pub struct SystemCtx {
    pub db_ctx: Arc<DbCtx>,
    pub memtable_flusher_handle: ArgonFsMemtableFlusherHandle,
    pub connector_handles: Vec<Box<dyn ConnectorHandle>>,
}
