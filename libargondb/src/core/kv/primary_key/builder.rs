use crate::kv::{primary_key::schema::Schema, value::Value};

pub struct PrimaryKeyBuilder<'a, W> {
    schema: &'a Schema,
    writer: W,
    current_index: usize,
}

impl<'a, W> PrimaryKeyBuilder<'a, W> {
    pub fn from_writer(schema: &'a Schema, writer: W) -> Self {
        Self {
            schema,
            writer,
            current_index: 0,
        }
    }
}

impl<'a> PrimaryKeyBuilder<'a, Vec<u8>> {
    pub fn new(schema: &'a Schema) -> Self {
        Self::from_writer(schema, Vec::new())
    }

    pub fn add_value(&mut self, value: &Value) -> Result<(), ()> {
        todo!()
    }
}
