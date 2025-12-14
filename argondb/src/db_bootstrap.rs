use std::sync::Arc;

use async_io::block_on;
use libargondb::{
    ArgonFs, ArgonFsConfig, Catalog,
    kv::{
        KVColumnFilter, KVPrimaryKeyMarker, KVRangeScan, KVTable,
        column_type::ColumnTypeCode,
        config::KVConfig,
        schema::{KVColumnSchema, KVTableSchema},
    },
};

use crate::db_ctx::DbCtx;

pub fn run_db_bootstrap() {
    block_on(async {
        let db_ctx = create_db_ctx();

        // create_argonsys_tables(&db_ctx).await;

        create_existing_tables(&db_ctx).await;
    });
}

fn create_db_ctx() -> Arc<DbCtx> {
    let catalog = Arc::new(Catalog::empty());
    let argon_fs = init_argon_fs();
    let kv_config = KVConfig::default();

    let db_ctx = Arc::new(DbCtx {
        kv_config,
        catalog,
        argon_fs,
    });

    db_ctx
}

fn init_argon_fs() -> Arc<ArgonFs> {
    let config = ArgonFsConfig::default();

    let argon_fs = ArgonFs::init(config).unwrap();

    Arc::new(argon_fs)
}

async fn create_existing_tables(db_ctx: &DbCtx) {
    let catalog = &db_ctx.catalog;

    let table = catalog.lookup_table_by_name("_argonsys_tables").unwrap();

    table
        .scan(KVRangeScan::new(
            KVPrimaryKeyMarker::Start,
            KVPrimaryKeyMarker::End,
            KVColumnFilter::All,
        ))
        .await;
    todo!()
}

// fn scan_and_load_table_sstables(db_ctx: &DbCtx, table_name: &str) -> Vec<Arc<KVSSTable>> {
// let scan_result = db_ctx.argon_fs.scan_table("_argonsys_tables");

// let sstables: Vec<Arc<Box<dyn KVSSTableReader + Send + Sync>>> = scan_result
//     .sstables
//     .into_iter()
//     .map(|p| db_ctx.argon_fs.open_sstable(p))
//     .collect();

// todo!()
// }
