use dashmap::DashMap;

use super::table::{Table, TableId, TableName};

pub struct Keyspace {
    tables: DashMap<TableId, Table>,
}

impl Keyspace {
    fn get_table_by_name(&self, name: TableName) -> Option<Table> {
        todo!()
    }

    fn get_table_by_id(&self, id: TableId) -> Option<Table> {
        todo!()
        // self.tables.get(id)
    }
}
