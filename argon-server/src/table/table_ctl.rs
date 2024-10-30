use std::sync::{atomic::AtomicPtr, Arc};

use dashmap::DashMap;
use md5::{Digest, Md5};
use uuid::Uuid;

use super::schema_ctl::SchemaCtl;

struct TableCtl {
    cf_store: DashMap<Uuid, TableCF>,
    schema: SchemaCtl,
}

impl TableCtl {
    fn new() -> Self {
        Self {
            cf_store: DashMap::new(),
            schema: SchemaCtl::new(),
        }
    }

    fn get_table_cf(&self, table: &str, cf: &str) {
        let mut hasher = Md5::new();
        hasher.update(table);
        hasher.update(cf);

        let uuid = uuid::Builder::from_md5_bytes(hasher.finalize().into()).into_uuid();
        // uuid
    }
}

struct TableCF {}
