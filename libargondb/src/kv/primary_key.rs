use crate::{
    ensure,
    kv::{
        column_type::{ColumnType, ColumnTypeCode},
        error::{KVConstructorError, KVRuntimeError},
        schema::KVTableSchema,
    },
};
use bytemuck::{bytes_of, from_bytes};
use std::cmp::Ordering;

/**
 * Stores schema of primary key
 */
#[derive(Debug)]
pub struct KVPrimaryKeySchema(Box<[u8]>);

impl KVPrimaryKeySchema {
    pub fn from_columns_schema(columns_schema: &KVTableSchema) -> Self {
        let column_count = columns_schema.primary_key.len();
        assert!(column_count > 0);
        assert!(column_count <= u8::MAX as usize);

        let mut buffer = Vec::<u8>::with_capacity(2 + column_count);
        buffer.push(column_count as u8);

        for column_id in &columns_schema.primary_key {
            assert_ne!(*column_id, 0);

            let column_schema = columns_schema.lookup_by_column_id(*column_id).unwrap();

            buffer.push(column_schema.column_type as u8);
        }

        Self(buffer.into_boxed_slice())
    }

    fn construct(data: Box<[u8]>) -> Result<Self, KVConstructorError> {
        ensure!(data.len() > 0, KVConstructorError::InvalidData);

        let column_count = data[0] as usize;
        ensure!(
            data.len() == (1 + column_count),
            KVConstructorError::InvalidData
        );

        Ok(Self(data))
    }

    fn column_count(&self) -> u8 {
        self.0[0]
    }

    fn column_type(&self, idx: usize) -> Result<impl ColumnType, KVRuntimeError> {
        ensure!(
            idx < self.column_count() as usize,
            KVRuntimeError::IndexOutOfBounds
        );

        let code = self.0[idx];
        ColumnTypeCode::type_for_code(code).ok_or(KVRuntimeError::DataMalformed)
    }
}

pub struct PrimaryKeyView<'a> {
    schema: &'a KVPrimaryKeySchema,
    key: &'a [u8],

    column_idx: usize,
    value_ptr: usize,
}

impl<'a> PrimaryKeyView<'a> {
    fn construct(
        schema: &'a KVPrimaryKeySchema,
        key: &'a [u8],
    ) -> Result<Self, KVConstructorError> {
        let column_count = schema.column_count() as usize;
        let column_idx = 0;

        let value_start = 2 * column_count;
        let value_ptr = value_start;

        Ok(Self {
            schema,
            key,
            column_idx,
            value_ptr,
        })
    }

    fn next_column(&mut self) -> Result<Option<(impl ColumnType, &[u8])>, KVRuntimeError> {
        let column_idx = self.column_idx;
        let value_ptr = self.value_ptr;

        if column_idx < self.schema.column_count() as usize {
            let size_ptr = 2 * column_idx;
            let value_size = *from_bytes::<u16>(&self.key[size_ptr..(size_ptr + 2)]) as usize;

            let col_type = self.schema.column_type(column_idx)?;
            let col_value = &self.key[value_ptr..(value_ptr + value_size)];

            self.value_ptr = value_ptr + value_size;
            self.column_idx = column_idx + 1;

            Ok(Some((col_type, col_value)))
        } else {
            Ok(None)
        }
    }
}

pub struct KVPrimaryKeyComparator;

impl KVPrimaryKeyComparator {
    pub fn cmp(
        schema: &KVPrimaryKeySchema,
        this: &[u8],
        that: &[u8],
    ) -> Result<Ordering, KVRuntimeError> {
        let column_count = schema.column_count();

        let mut this_key =
            PrimaryKeyView::construct(schema, this).map_err(|_| KVRuntimeError::DataMalformed)?;
        let mut that_key =
            PrimaryKeyView::construct(schema, that).map_err(|_| KVRuntimeError::DataMalformed)?;

        for _ in 0..column_count {
            let (Some(this_column), Some(that_column)) =
                (this_key.next_column()?, that_key.next_column()?)
            else {
                panic!("primary key comparison error")
            };

            match column_cmp(this_column.0, this_column.1, that_column.1) {
                Ordering::Equal => {}
                order => return Ok(order),
            }
        }

        Ok(Ordering::Equal)
    }

    pub fn eq(
        schema: &KVPrimaryKeySchema,
        this: &[u8],
        that: &[u8],
    ) -> Result<bool, KVRuntimeError> {
        let column_count = schema.column_count();

        let mut this_key =
            PrimaryKeyView::construct(schema, this).map_err(|_| KVRuntimeError::DataMalformed)?;
        let mut that_key =
            PrimaryKeyView::construct(schema, that).map_err(|_| KVRuntimeError::DataMalformed)?;

        for _ in 0..column_count {
            let (Some(this_column), Some(that_column)) =
                (this_key.next_column()?, that_key.next_column()?)
            else {
                panic!("primary key equality error")
            };

            if !column_eq(this_column.0, this_column.1, that_column.1) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

fn column_cmp<T: ColumnType>(_col_type: T, this: &[u8], that: &[u8]) -> Ordering {
    T::cmp(this, that)
}

fn column_eq<T: ColumnType>(_col_type: T, this: &[u8], that: &[u8]) -> bool {
    T::eq(this, that)
}

pub struct PrimaryKeyBuilder<'a> {
    schema: &'a KVPrimaryKeySchema,
    data: Vec<u8>,
    column_idx: u8,
}

impl<'a> PrimaryKeyBuilder<'a> {
    pub fn new(schema: &'a KVPrimaryKeySchema) -> Self {
        let column_count = schema.column_count() as usize;

        Self {
            schema,
            data: vec![0u8; 2 * column_count],
            column_idx: 0,
        }
    }

    pub fn add_value(&mut self, value: &[u8]) {
        assert!(value.len() < u16::MAX as usize);

        let value_size = value.len() as u16;

        let column_idx = self.column_idx as usize;
        self.column_idx += 1;

        self.data[(2 * column_idx)..(2 * (column_idx + 1))].copy_from_slice(bytes_of(&value_size));

        self.data.extend_from_slice(value);
    }

    pub fn build(self) -> Box<[u8]> {
        self.data.into_boxed_slice()
    }
}

pub enum KVPrimaryKeyMarker {
    Start,
    End,
    Key(Box<[u8]>),
}

struct PrimaryKeyMarkerComparator;

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
}

pub struct KVPrimaryKeyUtils;

impl KVPrimaryKeyUtils {
    pub fn size(primary_key: &[u8]) -> u16 {
        let pk_size = primary_key.len();
        assert!(pk_size <= u16::MAX as usize);

        pk_size as u16
    }
}
