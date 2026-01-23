use std::sync::Arc;

use crate::core::persistence::BoxPersistenceLayer;

use super::{Catalog, kv::KVInstance};

pub struct DbCtx {
    pub kv_instance: Arc<KVInstance>,
    pub catalog: Arc<Catalog>,
    pub persistence: Arc<BoxPersistenceLayer>,
}
