use crate::kv::{
    memtable::MemtableInsertError,
    mutation::{MutationType, StructuredMutation},
    primary_key::{PrimaryKeyBuilder, KVPrimaryKeySchema},
    table::KVTable,
    table_state::KVTableState,
};

#[derive(Debug)]
pub enum InsertOpError {
    InvalidColumnName,
    MissingPrimaryKey,
}

pub struct InsertOp {
    pub timestamp: u64,
    pub values: Vec<InsertOpValue>,
}

pub struct InsertOpValue {
    pub column_name: String,
    pub value: Box<[u8]>,
}

impl InsertOp {
    pub async fn execute(&self, table: &KVTable) -> Result<(), InsertOpError> {
        let table_state = table.load_state();

        let (prepared_values, primary_key) = self.prepare(&table_state)?;
        let mutations = self.prepare_mutations(&prepared_values, &primary_key)?;
        self.execute_insertions(table, &mutations).await;

        Ok(())
    }

    fn prepare(
        &self,
        table: &KVTableState,
    ) -> Result<(Vec<PreparedColumnValue>, Box<[u8]>), InsertOpError> {
        let mut prepared_values = Vec::<PreparedColumnValue>::new();

        for value in &self.values {
            let Some(column_schema) = table.columns_schema.lookup_by_name(&value.column_name)
            else {
                return Err(InsertOpError::InvalidColumnName);
            };

            prepared_values.push(PreparedColumnValue {
                column_id: column_schema.column_id,
                value: &value.value,
            });
        }

        prepared_values.sort_by(|a, b| a.column_id.cmp(&b.column_id));

        let pk_schema = KVPrimaryKeySchema::from_columns_schema(&table.columns_schema);
        let mut pk_builder = PrimaryKeyBuilder::new(&pk_schema);
        for column_id in &table.columns_schema.primary_key {
            let Ok(idx) = prepared_values
                .binary_search_by(|prepared_val| prepared_val.column_id.cmp(column_id))
            else {
                return Err(InsertOpError::MissingPrimaryKey);
            };

            pk_builder.add_value(prepared_values[idx].value);
        }
        let primary_key = pk_builder.build();

        Ok((prepared_values, primary_key))
    }

    fn prepare_mutations(
        &self,
        prepared_values: &Vec<PreparedColumnValue>,
        primary_key: &Box<[u8]>,
    ) -> Result<Vec<StructuredMutation>, InsertOpError> {
        let timestamp = self.timestamp;
        let mut mutations = Vec::<StructuredMutation>::new();

        for prepared_value in prepared_values {
            let column_id = prepared_value.column_id;
            let value = prepared_value.value.to_owned().into_boxed_slice();
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

    async fn execute_insertions(&self, table: &KVTable, mutations: &Vec<StructuredMutation>) {
        // TODO: Generally speaking, this code handling table state and getting new state after flush could be handled in a better way
        let mut table_state = table.load_state();
        for mutation in mutations {
            loop {
                match table_state.current_memtable.insert_mutation(&mutation) {
                    Ok(_) => {
                        break;
                    }
                    Err(MemtableInsertError::SizeExceeded)
                    | Err(MemtableInsertError::ReadOnlyMode) => {
                        table.flush_current_memtable().await;
                        table_state = table.load_state();
                    }
                }
            }
        }
    }
}

pub struct PreparedColumnValue<'a> {
    column_id: u16,
    value: &'a [u8],
}
