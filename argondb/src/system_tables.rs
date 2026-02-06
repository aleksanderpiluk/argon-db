use libargondb::kv::{
    Id, KVTableSchema, Name, column_type::ColumnTypeCode, schema::KVColumnSchema,
};

use crate::errors::{CriticalResult, OrCriticalError};

pub struct SystemTableNames;

impl SystemTableNames {
    pub const ARGONSYS_TABLES: Name<'static> =
        unsafe { Name::from_str_unchecked("_argonsys_tables") };
    pub const ARGONSYS_COLUMNS: Name<'static> =
        unsafe { Name::from_str_unchecked("_argonsys_columns") };
}

pub struct SystemTableIds;

impl SystemTableIds {
    pub const ARGONSYS_TABLES: Id<'static> = unsafe { Id::from_str_unchecked("_argsys_tbls") };
    pub const ARGONSYS_COLUMNS: Id<'static> = unsafe { Id::from_str_unchecked("_argsys_cols") };
}

pub struct SystemTableSchemas;

impl SystemTableSchemas {
    pub fn schema_argonsys_tables() -> CriticalResult<KVTableSchema> {
        KVTableSchema::build(
            vec![
                KVColumnSchema {
                    column_id: 1,
                    column_name: "table_id".to_string(),
                    column_type: ColumnTypeCode::Text,
                },
                KVColumnSchema {
                    column_id: 2,
                    column_name: "table_name".to_string(),
                    column_type: ColumnTypeCode::Text,
                },
                KVColumnSchema {
                    column_id: 3,
                    column_name: "primary_key".to_string(),
                    column_type: ColumnTypeCode::U16Array,
                },
            ],
            vec![1],
        )
        .ok_or_critical_err()
    }

    pub fn schema_argonsys_columns() -> CriticalResult<KVTableSchema> {
        KVTableSchema::build(
            vec![
                KVColumnSchema {
                    column_id: 1,
                    column_name: "table_id".to_string(),
                    column_type: ColumnTypeCode::Text,
                },
                KVColumnSchema {
                    column_id: 2,
                    column_name: "column_id".to_string(),
                    column_type: ColumnTypeCode::U16,
                },
                KVColumnSchema {
                    column_id: 3,
                    column_name: "column_name".to_string(),
                    column_type: ColumnTypeCode::Text,
                },
                KVColumnSchema {
                    column_id: 4,
                    column_name: "column_type".to_string(),
                    column_type: ColumnTypeCode::U16,
                },
            ],
            vec![1, 2],
        )
        .ok_or_critical_err()
    }
}

pub struct ArgonsysTablesColumns;

impl ArgonsysTablesColumns {
    pub const TABLE_ID: &'static str = "table_id";
    pub const TABLE_NAME: &'static str = "table_name";
    pub const PRIMARY_KEY: &'static str = "primary_key";
}

pub struct ArgonsysColumnsColumns;

impl ArgonsysColumnsColumns {
    pub const TABLE_ID: &'static str = "table_id";
    pub const COLUMN_ID: &'static str = "column_id";
    pub const COLUMN_NAME: &'static str = "column_name";
    pub const COLUMN_TYPE: &'static str = "column_type";
}
