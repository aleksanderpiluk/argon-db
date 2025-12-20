use libargondb::{
    ArgonFs, ArgonFsConfig, Catalog, DbCtx,
    kv::{
        KVColumnFilter, KVInstance, KVInstanceStateSnapshot, KVPrimaryKeyMarker, KVRangeScan,
        KVTable, KVTableId, KVTableName, KVTableSchema,
        column_type::{ColumnTypeCode, ColumnTypeText, ColumnTypeU16, ColumnTypeU16Array},
        config::KVConfig,
        schema::KVColumnSchema,
    },
    persistence::BoxPersistenceLayer,
};
use smol::block_on;
use std::{str::FromStr, sync::Arc, thread};

use crate::{
    errors::{CriticalError, CriticalResult, OrCriticalError},
    system_tables::{
        ArgonsysColumnsColumns, ArgonsysTablesColumns, SystemTableIds, SystemTableNames,
        SystemTableSchemas,
    },
};

pub fn run_init_thread() -> CriticalResult<Arc<DbCtx>> {
    println!("spawning init thread");

    let thread_handle = thread::spawn(init_thread);

    let init_thread_result = thread_handle
        .join()
        .map_err(|_| CriticalError::from_msg("failed to join on init thread"))?;

    let db_ctx = init_thread_result.map_err(|err| {
        CriticalError::from_msg_and_source("critical error while running init thread", err)
    })?;

    println!("init thread joined successfully");

    Ok(db_ctx)
}

fn init_thread() -> CriticalResult<Arc<DbCtx>> {
    println!("init thread - running initialization procedure");

    let db_ctx = init_db_ctx()?;
    println!("init thread - database context initialized");

    init_system_tables(&db_ctx)?;
    println!("init thread - system tables initialized");

    init_user_tables(&db_ctx)?;
    println!("init thread - user tables initialized");

    println!("init thread - initialization finished");
    Ok(db_ctx)
}

pub fn init_db_ctx() -> CriticalResult<Arc<DbCtx>> {
    let argon_fs_config = ArgonFsConfig::default();
    let argon_fs = ArgonFs::init(argon_fs_config).ok_or_critical_err()?;

    let persistence: Arc<BoxPersistenceLayer> = Arc::new(Box::new(argon_fs));

    let kv_config = KVConfig::default();
    let initial_snapshot = block_on(persistence.read_instance_snapshot())
        .ok_or_critical_err()?
        .unwrap_or_else(|| KVInstanceStateSnapshot::new());

    println!(
        "init thread - initializing instance with state snapshot {:?}",
        initial_snapshot
    );
    let kv_instance = Arc::new(KVInstance::new(kv_config, initial_snapshot));

    let catalog = Arc::new(Catalog::new());

    let db_ctx = Arc::new(DbCtx {
        kv_instance,
        catalog,
        persistence,
    });

    Ok(db_ctx)
}

pub fn init_system_tables(db_ctx: &DbCtx) -> CriticalResult<()> {
    add_table(
        db_ctx,
        &SystemTableIds::ARGONSYS_TABLES,
        &SystemTableNames::ARGONSYS_TABLES,
        SystemTableSchemas::schema_argonsys_tables()?,
    )?;

    add_table(
        db_ctx,
        &SystemTableIds::ARGONSYS_COLUMNS,
        &SystemTableNames::ARGONSYS_COLUMNS,
        SystemTableSchemas::schema_argonsys_columns()?,
    )?;

    Ok(())
}

pub fn init_user_tables(db_ctx: &DbCtx) -> CriticalResult<()> {
    let user_tables = block_on(scan_user_tables(db_ctx))?;
    println!("init thread - found {} user tables", user_tables.len());

    for (table_id, table_name, primary_key) in user_tables {
        println!("init thread - processing user table {}", table_name);

        let columns = block_on(scan_user_table_columns(db_ctx, &table_id))?;
        let columns = columns
            .into_iter()
            .map(|(column_id, column_name, column_type)| KVColumnSchema {
                column_id,
                column_name,
                column_type,
            })
            .collect();

        let table_schema =
            KVTableSchema::build(columns, primary_key.into_vec()).ok_or_critical_err()?;

        add_table(db_ctx, &table_id, &table_name, table_schema)?;
    }

    Ok(())
}

async fn scan_user_tables(
    db_ctx: &DbCtx,
) -> CriticalResult<Vec<(KVTableId<'_>, KVTableName<'_>, Box<[u16]>)>> {
    let argonsys_tables = db_ctx
        .catalog
        .lookup_table_by_name(&SystemTableNames::ARGONSYS_TABLES)
        .ok_or(CriticalError::from_msg("argonsys tables critical error"))?;

    let mut scan = argonsys_tables
        .scan(KVRangeScan::new(
            KVPrimaryKeyMarker::Start,
            KVPrimaryKeyMarker::End,
            KVColumnFilter::All,
        ))
        .await
        .ok_or_critical_err()?;

    let mut user_tables = Vec::<(KVTableId, KVTableName, Box<[u16]>)>::new();
    while let Some(row) = scan.next_row().await.ok_or_critical_err()? {
        let table_id_str = row
            .column_deserialized::<ColumnTypeText>(ArgonsysTablesColumns::TABLE_ID)
            .ok_or_critical_err()?;
        let table_id = KVTableId::from_str(&table_id_str).ok_or_critical_err()?;

        let table_name_str = row
            .column_deserialized::<ColumnTypeText>(ArgonsysTablesColumns::TABLE_NAME)
            .ok_or_critical_err()?;
        let table_name = KVTableName::from_str(&table_name_str).ok_or_critical_err()?;

        let primary_key = row
            .column_deserialized::<ColumnTypeU16Array>(ArgonsysTablesColumns::PRIMARY_KEY)
            .ok_or_critical_err()?;

        user_tables.push((table_id, table_name, primary_key));
    }

    Ok(user_tables)
}

async fn scan_user_table_columns(
    db_ctx: &DbCtx,
    table_id: &KVTableId<'_>,
) -> CriticalResult<Vec<(u16, String, ColumnTypeCode)>> {
    let argonsys_columns = db_ctx
        .catalog
        .lookup_table_by_name(&SystemTableNames::ARGONSYS_COLUMNS)
        .ok_or(CriticalError::from_msg("argonsys columns critical error"))?;

    // todo!("proper scan op");
    // TODO: Should be replaced with more advanced query
    let mut scan = argonsys_columns
        .scan(KVRangeScan::new(
            KVPrimaryKeyMarker::Start,
            KVPrimaryKeyMarker::End,
            KVColumnFilter::All,
        ))
        .await
        .ok_or_critical_err()?;

    let mut user_table_columns = Vec::<(u16, String, ColumnTypeCode)>::new();
    while let Some(row) = scan.next_row().await.ok_or_critical_err()? {
        let table_id_str = row
            .column_deserialized::<ColumnTypeText>(ArgonsysColumnsColumns::TABLE_ID)
            .ok_or_critical_err()?;
        let column_table_id = KVTableId::from_str(&table_id_str).ok_or_critical_err()?;

        let column_id = row
            .column_deserialized::<ColumnTypeU16>(ArgonsysColumnsColumns::COLUMN_ID)
            .ok_or_critical_err()?;

        let column_name_str = row
            .column_deserialized::<ColumnTypeText>(ArgonsysColumnsColumns::COLUMN_NAME)
            .ok_or_critical_err()?;

        let column_type_code = row
            .column_deserialized::<ColumnTypeU16>(ArgonsysColumnsColumns::COLUMN_TYPE)
            .ok_or_critical_err()?;
        let column_type_code =
            ColumnTypeCode::try_from(column_type_code as u8).ok_or_critical_err()?;

        if table_id.eq(&column_table_id) {
            user_table_columns.push((column_id, column_name_str, column_type_code));
        }
    }

    Ok(user_table_columns)
}

pub fn add_table(
    db_ctx: &DbCtx,
    table_id: &KVTableId,
    table_name: &KVTableName,
    table_schema: KVTableSchema,
) -> CriticalResult<()> {
    let scan_result = block_on(
        db_ctx
            .persistence
            .scan_for_sstables(table_id, &table_schema),
    );
    let sstables = scan_result.ok_or_critical_err()?;

    let table = Arc::new(KVTable::create(
        db_ctx.kv_instance.clone(),
        table_id.to_owned(),
        table_name.to_owned(),
        table_schema,
        sstables,
    ));
    table.open();

    db_ctx.catalog.add_table(table);

    Ok(())
}
