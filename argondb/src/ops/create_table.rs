use std::{collections::BTreeMap, str::FromStr, sync::Arc, vec};

use libargondb::{
    DbCtx,
    kv::{
        KVColumnValueBuilder, Table, Id, Name, KVTableSchema,
        column_type::ColumnTypeCode, schema::KVColumnSchema,
    },
};

use crate::{
    ops::insert_into::InsertIntoOp,
    system_tables::{self, SystemTableNames},
};

#[derive(Debug)]
pub enum CreateTableOpError {
    InvalidTableName,
    TooManyColumns,
    SchemaError,
    PrimaryKeyMissing,
    PrimaryKeyColumnsCountExceeded,
    PrimaryKeyInvalidColumn,
}

pub struct CreateTableOp {
    pub table_name: String,
    pub columns: Vec<CreateTableOpColumn>,
    pub primary_key: Vec<String>,
}

impl CreateTableOp {
    pub async fn execute(&self, db_ctx: &DbCtx) -> Result<Arc<Table>, CreateTableOpError> {
        let table_name = Name::from_str(&self.table_name)
            .map_err(|_| CreateTableOpError::InvalidTableName)?;

        if !(self.columns.len() < u16::MAX as usize) {
            return Err(CreateTableOpError::TooManyColumns);
        }

        if self.primary_key.len() == 0 {
            return Err(CreateTableOpError::PrimaryKeyMissing);
        }

        if self.primary_key.len() > u8::MAX as usize {
            return Err(CreateTableOpError::PrimaryKeyColumnsCountExceeded);
        }

        let mut next_column_id = 1u16;
        let mut columns = Vec::<KVColumnSchema>::new();
        let mut column_name_map = BTreeMap::<String, u16>::new();
        for column in &self.columns {
            let column_id = next_column_id;

            columns.push(KVColumnSchema {
                column_id,
                column_name: column.column_name.clone(),
                column_type: column.column_type,
            });
            column_name_map.insert(column.column_name.clone(), column_id);

            next_column_id += 1;
        }

        let mut primary_key = Vec::<u16>::new();
        for pk_name in &self.primary_key {
            let Some(column_id) = column_name_map.get(pk_name) else {
                return Err(CreateTableOpError::PrimaryKeyInvalidColumn);
            };

            primary_key.push(*column_id);
        }

        let table_schema = KVTableSchema::build(columns.clone(), primary_key.clone())
            .map_err(|_| CreateTableOpError::SchemaError)?;

        let table_id = Id::new_unique();

        let table = Arc::new(Table::create(
            db_ctx.kv_instance.clone(),
            table_id.clone(),
            table_name.clone(),
            table_schema,
            vec![],
        ));
        table.open();

        InsertIntoOp {
            table_name: SystemTableNames::ARGONSYS_TABLES.to_string(),
            values: vec![
                (
                    "table_id".into(),
                    KVColumnValueBuilder::text(table_id.to_string()),
                ),
                (
                    "table_name".into(),
                    KVColumnValueBuilder::text(table_name.to_string()),
                ),
                (
                    "primary_key".into(),
                    KVColumnValueBuilder::u16_array(primary_key),
                ),
            ],
        }
        .execute(db_ctx)
        .await
        .unwrap();

        for column in columns {
            InsertIntoOp {
                table_name: SystemTableNames::ARGONSYS_COLUMNS.to_string(),
                values: vec![
                    (
                        "table_id".into(),
                        KVColumnValueBuilder::text(table_id.to_string()),
                    ),
                    (
                        "column_id".into(),
                        KVColumnValueBuilder::u16(column.column_id),
                    ),
                    (
                        "column_name".into(),
                        KVColumnValueBuilder::text(column.column_name),
                    ),
                    (
                        "column_type".into(),
                        KVColumnValueBuilder::u16(column.column_type as u16),
                    ),
                ],
            }
            .execute(db_ctx)
            .await
            .unwrap();
        }

        db_ctx.catalog.add_table(table.clone());

        Ok(table)
    }
}

pub struct CreateTableOpColumn {
    pub column_name: String,
    pub column_type: ColumnTypeCode,
}
