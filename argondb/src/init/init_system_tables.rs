use std::sync::Arc;

use libargondb::kv::{KVTable, schema::KVTableSchema};

use crate::{
    db_ctx::DbCtx,
    errors::{CriticalResult, OrCriticalError},
    init::init_system_tables::system_table_schemas::{
        schema_argonsys_columns, schema_argonsys_tables,
    },
};

pub fn init_system_tables(db_ctx: &DbCtx) -> CriticalResult<()> {
    let argonsys_tables_schema = schema_argonsys_tables()?;
    add_table(db_ctx, argonsys_tables_schema)?;

    let argonsys_columns_schema = schema_argonsys_columns()?;
    add_table(db_ctx, argonsys_columns_schema)?;

    Ok(())
}

fn add_table(db_ctx: &DbCtx, table_schema: KVTableSchema) -> CriticalResult<()> {
    let kv_config = db_ctx.kv_config.clone();
    let sstables = db_ctx
        .argon_fs
        .scan_for_sstables(&table_schema)
        .ok_or_critical_err()?;

    let table = KVTable::create(kv_config, table_schema, sstables);

    db_ctx.catalog.add_table(Arc::new(table));

    Ok(())
}

mod system_table_schemas {
    use libargondb::kv::{
        column_type::ColumnTypeCode,
        schema::{KVColumnSchema, KVTableSchema},
    };

    use crate::errors::{CriticalResult, OrCriticalError};

    pub fn schema_argonsys_tables() -> CriticalResult<KVTableSchema> {
        KVTableSchema::build(
            "_argonsys_tables".to_string(),
            "_argonsys_tables".to_string(),
            vec![
                KVColumnSchema {
                    column_id: 1,
                    column_name: "table_id".to_string(),
                    column_type: ColumnTypeCode::Bytes,
                },
                KVColumnSchema {
                    column_id: 2,
                    column_name: "table_name".to_string(),
                    column_type: ColumnTypeCode::Bytes,
                },
            ],
            vec![1],
        )
        .ok_or_critical_err()
    }

    pub fn schema_argonsys_columns() -> CriticalResult<KVTableSchema> {
        KVTableSchema::build(
            "_argonsys_columns".to_string(),
            "_argonsys_columns".to_string(),
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
        .ok_or_critical_err()
    }
}
