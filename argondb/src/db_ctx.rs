use std::sync::Arc;

use libargondb::{ArgonFs, Catalog};

pub struct DbCtx {
    pub catalog: Arc<Catalog>,
    pub argon_fs: Arc<ArgonFs>,
}
