use crate::kv::{
    KVRuntimeError,
    primary_key::{PrimaryKeyData, schema::PrimaryKeySchema},
    value::Value,
};

pub struct PrimaryKeyView<'a> {
    schema: &'a PrimaryKeySchema,
    data: &'a PrimaryKeyData<'a>,

    column_index: usize,
    data_offset: usize,
}

impl<'a> PrimaryKeyView<'a> {
    pub fn new(schema: &'a PrimaryKeySchema, data: &'a PrimaryKeyData<'a>) -> Self {
        let initial_data_offset = schema.column_count() as usize * 2;

        Self {
            schema,
            data,
            column_index: 0,
            data_offset: initial_data_offset,
        }
    }

    pub fn next_column(&mut self) -> Result<Option<Value<'a>>, KVRuntimeError> {
        let column_index = self.column_index;
        let data_offset = self.data_offset;

        let column_count = self.schema.column_count() as usize;
        if column_index > column_count {
            return Ok(None);
        }

        let value_size_offset = column_index * 2;
        let value_size = u16::from_le_bytes(
            self.data.0[value_size_offset..(value_size_offset + 2)]
                .try_into()
                .unwrap(),
        ) as usize;

        let value_type = self.schema.get_column(column_index)?;
        let data = &self.data.0[data_offset..(data_offset + value_size)];
        self.data_offset = data_offset + value_size;
        self.column_index = self.column_index + 1;

        Ok(Some(Value::new(value_type, data)?))
    }
}
