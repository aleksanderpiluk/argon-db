use std::{
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use libargondb::{
    DbCtx,
    kv::{
        KVColumnValue, Name, Table,
        mutation::{MutationType, StructuredMutation},
        primary_key::{KVPrimaryKeySchema, PrimaryKeyBuilder},
    },
};

#[derive(Debug)]
pub enum InsertOpError {
    InvalidTableName,
    InvalidColumnName,
    MissingPrimaryKey,
    TableNotFound,
}

pub struct InsertIntoOp {
    pub table_name: String,
    pub values: Vec<(String, Box<dyn KVColumnValue + Send + Sync + 'static>)>,
}

impl InsertIntoOp {
    pub async fn execute(&self, db_ctx: &DbCtx) -> Result<(), InsertOpError> {
        let table_name =
            Name::from_str(&self.table_name).map_err(|_| InsertOpError::InvalidTableName)?;

        let table = db_ctx
            .catalog
            .lookup_table_by_name(&table_name)
            .ok_or(InsertOpError::TableNotFound)?;

        let (prepared_values, primary_key) = self.prepare(&table)?;
        let mutations = self.prepare_mutations(&prepared_values, &primary_key)?;
        self.execute_insertions(&table, &mutations).await;

        Ok(())
    }

    fn prepare(
        &self,
        table: &Table,
    ) -> Result<(Vec<PreparedColumnValue>, Box<[u8]>), InsertOpError> {
        let mut prepared_values = Vec::<PreparedColumnValue>::new();

        for (column_name, column_value) in &self.values {
            let Some(column_schema) = table.table_schema.lookup_by_name(column_name) else {
                return Err(InsertOpError::InvalidColumnName);
            };

            prepared_values.push(PreparedColumnValue {
                column_id: column_schema.column_id,
                value: column_value.serialize().unwrap(),
            });
        }

        prepared_values.sort_by(|a, b| a.column_id.cmp(&b.column_id));

        let pk_schema = KVPrimaryKeySchema::from_table_schema(&table.table_schema);
        let mut pk_builder = PrimaryKeyBuilder::new(&pk_schema);
        for column_id in &table.table_schema.primary_key {
            let Ok(idx) = prepared_values
                .binary_search_by(|prepared_val| prepared_val.column_id.cmp(column_id))
            else {
                return Err(InsertOpError::MissingPrimaryKey);
            };

            pk_builder.add_value(&prepared_values[idx].value);
        }
        let primary_key = pk_builder.build();

        Ok((prepared_values, primary_key))
    }

    fn prepare_mutations(
        &self,
        prepared_values: &Vec<PreparedColumnValue>,
        primary_key: &Box<[u8]>,
    ) -> Result<Vec<StructuredMutation>, InsertOpError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let mut mutations = Vec::<StructuredMutation>::new();

        for prepared_value in prepared_values {
            let column_id = prepared_value.column_id;
            let value = prepared_value.value.clone();
            let primary_key = primary_key.clone();

            let mutation = StructuredMutation::try_from(
                timestamp,
                column_id,
                MutationType::Put,
                primary_key,
                value,
            )
            .unwrap();

            mutations.push(mutation);
        }

        Ok(mutations)
    }

    async fn execute_insertions(&self, table: &Arc<Table>, mutations: &Vec<StructuredMutation>) {
        // TODO: Generally speaking, this code handling table state and getting new state after flush could be handled in a better way
        table.insert_mutations(mutations).await.unwrap();
        // let mut table_state = table.load_state();
        // for mutation in mutations {
        //     loop {
        //         match table_state.current_memtable.insert_mutation(&mutation) {
        //             Ok(_) => {
        //                 break;
        //             }
        //             Err(MemtableInsertError::SizeExceeded)
        //             | Err(MemtableInsertError::ReadOnlyMode) => {
        //                 table.flush_current_memtable().await;
        //                 table_state = table.load_state();
        //             }
        //         }
        //     }
        // }
    }
}

pub struct PreparedColumnValue {
    column_id: u16,
    value: Box<[u8]>,
}
