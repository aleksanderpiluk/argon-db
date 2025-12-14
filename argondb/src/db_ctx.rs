use std::sync::Arc;

use libargondb::{ArgonFs, Catalog, kv::config::KVConfig};

pub struct DbCtx {
    pub kv_config: KVConfig,
    pub catalog: Arc<Catalog>,
    pub argon_fs: Arc<ArgonFs>,
}
