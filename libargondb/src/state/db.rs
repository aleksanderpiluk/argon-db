use crate::state::table::TableStateSnapshot;

pub struct DbStateSnapshot {
    tables: Box<[TableStateSnapshot]>,
}

impl DbStateSnapshot {
    fn table_lookup_by_name(&self, table_name: String) -> Option<&TableStateSnapshot> {
        match self
            .tables
            .binary_search_by(|t| t.table_name.cmp(&table_name))
        {
            Ok(idx) => Some(&self.tables[idx]),
            Err(_) => None,
        }
    }
}

impl DbStateSnapshot {
    fn add_table(snapshot: &Self, table: TableStateSnapshot) -> Result<Self, ()> {
        let idx = match snapshot
            .tables
            .binary_search_by(|t| t.table_name.cmp(&table.table_name))
        {
            Ok(_) => return Err(()),
            Err(idx) => idx,
        };

        let mut tables = Vec::with_capacity(snapshot.tables.len() + 1);
        tables.extend_from_slice(&snapshot.tables[0..idx]);
        tables.push(table);
        tables.extend_from_slice(&snapshot.tables[idx..]);

        Ok(Self {
            tables: tables.into_boxed_slice(),
        })
    }
}
