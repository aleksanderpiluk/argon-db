use std::{collections::BTreeMap, sync::Arc};

use crate::kv::{
    KVTable,
    column_type::ColumnTypeCode,
    config::KVConfig,
    schema::{KVColumnSchema, KVTableSchema},
};

#[derive(Debug)]
pub enum CreateTableOpError {
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
    pub fn execute(&self) -> Result<Arc<KVTable>, CreateTableOpError> {
        assert!(self.columns.len() < u16::MAX as usize);

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

        let columns_schema = KVTableSchema {
            columns,
            column_name_map,
            primary_key,
        };

        let config = KVConfig::default();

        Ok(Arc::new(KVTable::create(config, columns_schema)))
    }
}

pub struct CreateTableOpColumn {
    pub column_name: String,
    pub column_type: ColumnTypeCode,
}
