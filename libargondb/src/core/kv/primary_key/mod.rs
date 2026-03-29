mod builder;
mod comparator;
mod schema;
mod view;

pub use comparator::PrimaryKeyComparator;
pub use schema::Schema;

pub struct PrimaryKey<'a>(std::borrow::Cow<'a, [u8]>);

// #[derive(Debug, Clone)]
// pub enum KVPrimaryKeyMarker {
//     Start,
//     End,
//     Key(Box<[u8]>),
// }

// pub struct PrimaryKeyMarkerComparator;

// impl PrimaryKeyMarkerComparator {
//     pub fn cmp(
//         schema: &KVPrimaryKeySchema,
//         this: &KVPrimaryKeyMarker,
//         that: &KVPrimaryKeyMarker,
//     ) -> Result<Ordering, KVRuntimeError> {
//         if let KVPrimaryKeyMarker::Start = this {
//             if let KVPrimaryKeyMarker::Start = that {
//                 return Ok(Ordering::Equal);
//             } else {
//                 return Ok(Ordering::Less);
//             }
//         }

//         if let KVPrimaryKeyMarker::End = this {
//             if let KVPrimaryKeyMarker::End = that {
//                 return Ok(Ordering::Equal);
//             } else {
//                 return Ok(Ordering::Greater);
//             }
//         }

//         if let KVPrimaryKeyMarker::Key(this_key) = this {
//             return match that {
//                 KVPrimaryKeyMarker::Key(that_key) => {
//                     KVPrimaryKeyComparator::cmp(schema, &this_key, &that_key)
//                 }
//                 KVPrimaryKeyMarker::Start => Ok(Ordering::Greater),
//                 KVPrimaryKeyMarker::End => Ok(Ordering::Less),
//             };
//         }

//         panic!("PrimaryKeyMarkerComparator fatal error");
//     }

//     pub fn cmp_with_key(
//         schema: &KVPrimaryKeySchema,
//         this: &KVPrimaryKeyMarker,
//         that: &[u8],
//     ) -> Result<Ordering, KVRuntimeError> {
//         match this {
//             KVPrimaryKeyMarker::Start => Ok(Ordering::Less),
//             KVPrimaryKeyMarker::End => Ok(Ordering::Greater),
//             KVPrimaryKeyMarker::Key(this) => KVPrimaryKeyComparator::cmp(schema, this, that),
//         }
//     }
// }

// pub struct KVPrimaryKeyUtils;

// impl KVPrimaryKeyUtils {
//     pub fn size(primary_key: &[u8]) -> u16 {
//         let pk_size = primary_key.len();
//         assert!(pk_size <= u16::MAX as usize);

//         pk_size as u16
//     }

//     pub fn debug_fmt(schema: &KVTableSchema, key: &[u8]) -> Result<String, ()> {
//         let pk_schema = KVPrimaryKeySchema::from_table_schema(&schema);
//         let mut pk_view = PrimaryKeyView::construct(&pk_schema, key).map_err(|_| ())?;

//         let mut out = String::from("|");

//         while let Some((column_type, column_value)) = pk_view.next_column().map_err(|_| ())? {
//             out += &format!(
//                 "{}|",
//                 KVColumnTypeUtils::debug_fmt(column_type.code(), column_value)
//             );
//         }

//         Ok(out)
//     }
// }

// pub struct KVPrimaryKeyMarkerUtils;

// impl KVPrimaryKeyMarkerUtils {
//     pub fn debug_fmt(schema: &KVTableSchema, key: &KVPrimaryKeyMarker) -> Result<String, ()> {
//         let pk_schema = KVPrimaryKeySchema::from_table_schema(&schema);

//         match key {
//             KVPrimaryKeyMarker::Start => Ok("Start".to_string()),
//             KVPrimaryKeyMarker::End => Ok("End".to_string()),
//             KVPrimaryKeyMarker::Key(key) => {
//                 let mut pk_view = PrimaryKeyView::construct(&pk_schema, key).map_err(|_| ())?;

//                 let mut out = String::from("|");

//                 while let Some((column_type, column_value)) =
//                     pk_view.next_column().map_err(|_| ())?
//                 {
//                     out += &format!(
//                         "{}|",
//                         KVColumnTypeUtils::debug_fmt(column_type.code(), column_value)
//                     );
//                 }

//                 Ok(out)
//             }
//         }
//     }
// }
