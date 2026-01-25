mod builder;
mod comparator;
mod schema;
mod view;

pub use builder::PrimaryKeyBuilder;
pub use schema::PrimaryKeySchema;

use crate::kv::{
    KVRuntimeErrorKind,
    column_type::{ColumnType, ColumnTypeCode, KVColumnTypeUtils},
    core::Comparator,
    error::{KVConstructorError, KVRuntimeError},
    primary_key::view::PrimaryKeyView,
    schema::KVTableSchema,
};
use std::{borrow::Cow, cmp::Ordering};

struct PrimaryKeyData<'a>(Cow<'a, [u8]>);

// #[derive(Debug, Clone)]
// pub struct KVPrimaryKeySchema(Box<[u8]>);

// impl KVPrimaryKeySchema {
//     pub fn from_table_schema(columns_schema: &KVTableSchema) -> Self {
//         let column_count = columns_schema.primary_key.len();
//         assert!(column_count > 0);
//         assert!(column_count <= u8::MAX as usize);

//         let mut buffer = Vec::<u8>::with_capacity(1 + column_count);
//         buffer.push(column_count as u8);

//         for column_id in &columns_schema.primary_key {
//             assert_ne!(*column_id, 0);

//             let column_schema = columns_schema.lookup_by_column_id(*column_id).unwrap();

//             buffer.push(column_schema.column_type as u8);
//         }

//         Self(buffer.into_boxed_slice())
//     }

//     fn column_count(&self) -> u8 {
//         self.0[0]
//     }

//     fn column_type(&self, idx: usize) -> Result<Box<dyn ColumnType>, KVRuntimeError> {
//         ensure!(
//             idx < self.column_count() as usize,
//             KVRuntimeError::with_msg(
//                 KVRuntimeErrorKind::IndexOutOfBounds,
//                 format!("primary key column index {} out of bounds", idx)
//             )
//         );

//         let code = self.0[1 + idx];
//         ColumnTypeCode::type_for_code(code)
//     }
// }

#[derive(Debug, Clone)]
pub enum KVPrimaryKeyMarker {
    Start,
    End,
    Key(Box<[u8]>),
}

pub struct PrimaryKeyMarkerComparator;

impl PrimaryKeyMarkerComparator {
    pub fn cmp(
        schema: &KVPrimaryKeySchema,
        this: &KVPrimaryKeyMarker,
        that: &KVPrimaryKeyMarker,
    ) -> Result<Ordering, KVRuntimeError> {
        if let KVPrimaryKeyMarker::Start = this {
            if let KVPrimaryKeyMarker::Start = that {
                return Ok(Ordering::Equal);
            } else {
                return Ok(Ordering::Less);
            }
        }

        if let KVPrimaryKeyMarker::End = this {
            if let KVPrimaryKeyMarker::End = that {
                return Ok(Ordering::Equal);
            } else {
                return Ok(Ordering::Greater);
            }
        }

        if let KVPrimaryKeyMarker::Key(this_key) = this {
            return match that {
                KVPrimaryKeyMarker::Key(that_key) => {
                    KVPrimaryKeyComparator::cmp(schema, &this_key, &that_key)
                }
                KVPrimaryKeyMarker::Start => Ok(Ordering::Greater),
                KVPrimaryKeyMarker::End => Ok(Ordering::Less),
            };
        }

        panic!("PrimaryKeyMarkerComparator fatal error");
    }

    pub fn cmp_with_key(
        schema: &KVPrimaryKeySchema,
        this: &KVPrimaryKeyMarker,
        that: &[u8],
    ) -> Result<Ordering, KVRuntimeError> {
        match this {
            KVPrimaryKeyMarker::Start => Ok(Ordering::Less),
            KVPrimaryKeyMarker::End => Ok(Ordering::Greater),
            KVPrimaryKeyMarker::Key(this) => KVPrimaryKeyComparator::cmp(schema, this, that),
        }
    }
}

pub struct KVPrimaryKeyUtils;

impl KVPrimaryKeyUtils {
    pub fn size(primary_key: &[u8]) -> u16 {
        let pk_size = primary_key.len();
        assert!(pk_size <= u16::MAX as usize);

        pk_size as u16
    }

    pub fn debug_fmt(schema: &KVTableSchema, key: &[u8]) -> Result<String, ()> {
        let pk_schema = KVPrimaryKeySchema::from_table_schema(&schema);
        let mut pk_view = PrimaryKeyView::construct(&pk_schema, key).map_err(|_| ())?;

        let mut out = String::from("|");

        while let Some((column_type, column_value)) = pk_view.next_column().map_err(|_| ())? {
            out += &format!(
                "{}|",
                KVColumnTypeUtils::debug_fmt(column_type.code(), column_value)
            );
        }

        Ok(out)
    }
}

pub struct KVPrimaryKeyMarkerUtils;

impl KVPrimaryKeyMarkerUtils {
    pub fn debug_fmt(schema: &KVTableSchema, key: &KVPrimaryKeyMarker) -> Result<String, ()> {
        let pk_schema = KVPrimaryKeySchema::from_table_schema(&schema);

        match key {
            KVPrimaryKeyMarker::Start => Ok("Start".to_string()),
            KVPrimaryKeyMarker::End => Ok("End".to_string()),
            KVPrimaryKeyMarker::Key(key) => {
                let mut pk_view = PrimaryKeyView::construct(&pk_schema, key).map_err(|_| ())?;

                let mut out = String::from("|");

                while let Some((column_type, column_value)) =
                    pk_view.next_column().map_err(|_| ())?
                {
                    out += &format!(
                        "{}|",
                        KVColumnTypeUtils::debug_fmt(column_type.code(), column_value)
                    );
                }

                Ok(out)
            }
        }
    }
}
