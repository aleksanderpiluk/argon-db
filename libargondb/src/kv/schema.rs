use std::{
    collections::{BTreeMap, HashSet},
    fmt::Debug,
};

use crate::{
    ensure,
    kv::{
        KVLimits,
        column_type::{self, ColumnType, ColumnTypeCode},
    },
};

#[derive(Clone)]
pub struct KVTableSchema {
    pub table_id: String,
    pub table_name: String,
    pub columns: Vec<KVColumnSchema>,

    /** Contains IDs of columns being part of primary key in correct order */
    pub primary_key: Vec<u16>,
    pub column_name_map: BTreeMap<String, u16>,
}

impl KVTableSchema {
    pub fn build(
        table_id: String,
        table_name: String,
        columns: Vec<KVColumnSchema>,
        primary_key: Vec<u16>,
    ) -> Result<Self, KVTableSchemaBuildError> {
        ensure!(columns.len() > 0, KVTableSchemaBuildError::ColumnCountZero);
        ensure!(
            columns.len() < KVLimits::TABLE_MAX_COLUMNS,
            KVTableSchemaBuildError::ColumnCountExceeded
        );

        ensure!(
            primary_key.len() > 0,
            KVTableSchemaBuildError::PrimaryKeyColumnCountZero
        );
        ensure!(
            primary_key.len() < KVLimits::PRIMARY_KEY_MAX_COLUMNS,
            KVTableSchemaBuildError::PrimaryKeyColumnCountExceeded
        );

        let mut column_name_map: BTreeMap<String, u16> = BTreeMap::new();
        let mut last_column_id: u16 = 0;

        for column in &columns {
            let column_id = column.column_id;
            let column_name = column.column_name.clone();

            ensure!(
                column_id > last_column_id, // Checks both next column ids are growing and are greater than 0
                KVTableSchemaBuildError::ColumnsSchemaInvalid
            );
            last_column_id = column_id;

            ensure!(
                !column_name_map.contains_key(&column_name),
                KVTableSchemaBuildError::ColumnsSchemaInvalid
            );

            column_name_map.insert(column_name, column.column_id);
        }

        let mut primary_key_column_ids_set: HashSet<u16> =
            HashSet::with_capacity(primary_key.len());
        for column_id in &primary_key {
            ensure!(
                columns
                    .binary_search_by(|x| x.column_id.cmp(column_id))
                    .is_ok(),
                KVTableSchemaBuildError::PrimaryKeyInvalid
            );

            ensure!(
                !primary_key_column_ids_set.contains(column_id),
                KVTableSchemaBuildError::PrimaryKeyInvalid
            );

            primary_key_column_ids_set.insert(*column_id);
        }

        Ok(Self {
            table_id,
            table_name,
            columns,
            primary_key,
            column_name_map,
        })
    }

    pub fn columns_count(&self) -> u16 {
        let len = self.columns.len();
        assert!(len <= u16::MAX as usize);

        len as u16
    }

    pub fn lookup_by_column_id(&self, column_id: u16) -> Option<&KVColumnSchema> {
        assert_ne!(column_id, 0);

        match self
            .columns
            .binary_search_by(|column_schema| column_schema.column_id.cmp(&column_id))
        {
            Ok(idx) => Some(&self.columns[idx]),
            Err(_) => None,
        }
    }

    pub fn lookup_by_name(&self, column_name: &String) -> Option<&KVColumnSchema> {
        let Some(column_id) = self.column_name_map.get(column_name) else {
            return None;
        };

        let result = self.lookup_by_column_id(*column_id);
        assert!(result.is_some());

        result
    }
}

impl Debug for KVTableSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KVTableSchema")
            .field("columns", &self.columns)
            .field("primary_key", &self.primary_key)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct KVColumnSchema {
    pub column_id: u16,
    pub column_name: String,
    pub column_type: ColumnTypeCode,
}

#[derive(Debug)]
pub enum KVTableSchemaBuildError {
    ColumnCountZero,
    ColumnCountExceeded,
    ColumnsSchemaInvalid,
    PrimaryKeyColumnCountZero,
    PrimaryKeyColumnCountExceeded,
    PrimaryKeyInvalid,
}
