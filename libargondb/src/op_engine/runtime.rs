use crate::{catalog::client::SchemaClient, storage::client::StorageClient};

struct RuntimeCtx {
    schema: SchemaClient,
    storage: StorageClient,
}

struct Runtime {
    ctx: RuntimeCtx,
}

impl Runtime {}
