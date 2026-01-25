use crate::kv::{primary_key::schema::PrimaryKeySchema, value::Value};

pub struct PrimaryKeyBuilder<'a, W> {
    schema: &'a PrimaryKeySchema,
    writer: W,
    current_index: usize,
}

impl<'a, W> PrimaryKeyBuilder<'a, W> {
    pub fn from_writer(schema: &'a PrimaryKeySchema, writer: W) -> Self {
        Self {
            schema,
            writer,
            current_index: 0,
        }
    }
}

impl<'a> PrimaryKeyBuilder<'a, Vec<u8>> {
    pub fn new(schema: &'a PrimaryKeySchema) -> Self {
        Self::from_writer(schema, Vec::new())
    }

    pub fn add_value(&mut self, value: &Value) -> Result<(), ()> {
        todo!()
    }
}

// pub struct PrimaryKeyBuilder<'a> {
//     schema: &'a KVPrimaryKeySchema,
//     data: Vec<u8>,
//     column_idx: u8,
// }

// impl<'a> PrimaryKeyBuilder<'a> {
//     pub fn new(schema: &'a KVPrimaryKeySchema) -> Self {
//         let column_count = schema.column_count() as usize;

//         Self {
//             schema,
//             data: vec![0u8; 2 * column_count],
//             column_idx: 0,
//         }
//     }

//     pub fn add_value(&mut self, value: &[u8]) {
//         assert!(value.len() < u16::MAX as usize);

//         let value_size = value.len() as u16;

//         let column_idx = self.column_idx as usize;
//         self.column_idx += 1;

//         self.data[(2 * column_idx)..(2 * (column_idx + 1))].copy_from_slice(bytes_of(&value_size));

//         self.data.extend_from_slice(value);
//     }

//     pub fn build(self) -> Box<[u8]> {
//         self.data.into_boxed_slice()
//     }
// }
