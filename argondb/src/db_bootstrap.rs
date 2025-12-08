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

        create_argonsys_tables(&db_ctx).await;

        create_existing_tables(&db_ctx).await;
    });
}

fn create_db_ctx() -> Arc<DbCtx> {
    let catalog = Arc::new(Catalog::empty());
    let argon_fs = init_argon_fs();

    let db_ctx = Arc::new(DbCtx { catalog, argon_fs });

    db_ctx
}

fn init_argon_fs() -> Arc<ArgonFs> {
    let config = ArgonFsConfig::default();

    let argon_fs = ArgonFs::init(config).unwrap();

    Arc::new(argon_fs)
}

async fn create_argonsys_tables(db_ctx: &DbCtx) {
    let catalog = &db_ctx.catalog;
    let argon_fs = &db_ctx.argon_fs;

    let kv_config = KVConfig::default();

    let sstables = argon_fs.scan_sstables("_argonsys_tables").await.unwrap();
    let argonsys_tables_table = KVTable::create(
        kv_config.clone(),
        get_argonsys_tables_table_schema(),
        sstables,
    );
    catalog.add_table(
        "_argonsys_tables".to_string(),
        Arc::new(argonsys_tables_table),
    );

    let sstables = argon_fs.scan_sstables("_argonsys_columns").await.unwrap();
    let argonsys_columns_table = KVTable::create(
        kv_config.clone(),
        get_argonsys_columns_table_schema(),
        sstables,
    );
    catalog.add_table(
        "_argonsys_columns".to_string(),
        Arc::new(argonsys_columns_table),
    );
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

fn get_argonsys_tables_table_schema() -> KVTableSchema {
    KVTableSchema::build(
        vec![KVColumnSchema {
            column_id: 1,
            column_name: "table_name".to_string(),
            column_type: ColumnTypeCode::Bytes,
        }],
        vec![1],
    )
    .unwrap()
}

fn get_argonsys_columns_table_schema() -> KVTableSchema {
    KVTableSchema::build(
        vec![
            KVColumnSchema {
                column_id: 1,
                column_name: "table_name".to_string(),
                column_type: ColumnTypeCode::Bytes,
            },
            KVColumnSchema {
                column_id: 2,
                column_name: "column_id".to_string(),
                column_type: ColumnTypeCode::Bytes,
            },
            KVColumnSchema {
                column_id: 3,
                column_name: "column_name".to_string(),
                column_type: ColumnTypeCode::Bytes,
            },
        ],
        vec![1, 2],
    )
    .unwrap()
}
