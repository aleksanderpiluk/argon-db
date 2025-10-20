use std::collections::BTreeMap;

use crate::kv::column_type::{self, ColumnType, ColumnTypeCode};

#[derive(Clone, Debug)]
pub struct KVColumnsSchema {
    pub columns: Vec<KVColumnSchema>,

    /** Contains IDs of columns being part of primary key in correct order */
    pub primary_key: Vec<u16>,
    pub column_name_map: BTreeMap<String, u16>,
}

impl KVColumnsSchema {
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

#[derive(Debug, Clone)]
pub struct KVColumnSchema {
    pub column_id: u16,
    pub column_name: String,
    pub column_type: ColumnTypeCode,
}

impl KVColumnSchema {
    pub fn column_type(&self) -> impl ColumnType {
        todo!();
        column_type::Bytes
    }

    pub fn column_name(&self) -> &str {
        todo!()
    }
}
