use crate::{
    ensure,
    kv::{
        KVRuntimeErrorKind,
        column_type::{ColumnType, ColumnTypeCode, KVColumnTypeUtils},
        error::{KVConstructorError, KVRuntimeError},
        schema::KVTableSchema,
    },
};
use bytemuck::{bytes_of, from_bytes};
use std::cmp::Ordering;

/// Stores schema of primary key  
#[derive(Debug, Clone)]
pub struct KVPrimaryKeySchema(Box<[u8]>);

impl KVPrimaryKeySchema {
    pub fn from_table_schema(columns_schema: &KVTableSchema) -> Self {
        let column_count = columns_schema.primary_key.len();
        assert!(column_count > 0);
        assert!(column_count <= u8::MAX as usize);

        let mut buffer = Vec::<u8>::with_capacity(1 + column_count);
        buffer.push(column_count as u8);

        for column_id in &columns_schema.primary_key {
            assert_ne!(*column_id, 0);

            let column_schema = columns_schema.lookup_by_column_id(*column_id).unwrap();

            buffer.push(column_schema.column_type as u8);
        }

        Self(buffer.into_boxed_slice())
    }

    fn column_count(&self) -> u8 {
        self.0[0]
    }

    fn column_type(&self, idx: usize) -> Result<Box<dyn ColumnType>, KVRuntimeError> {
        ensure!(
            idx < self.column_count() as usize,
            KVRuntimeError::with_msg(
                KVRuntimeErrorKind::IndexOutOfBounds,
                format!("primary key column index {} out of bounds", idx)
            )
        );

        let code = self.0[1 + idx];
        ColumnTypeCode::type_for_code(code)
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

    fn next_column(&mut self) -> Result<Option<(Box<dyn ColumnType>, &[u8])>, KVRuntimeError> {
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

        let mut this_key = PrimaryKeyView::construct(schema, this)
            .map_err(|e| KVRuntimeError::with_source(KVRuntimeErrorKind::DataMalformed, e))?;
        let mut that_key = PrimaryKeyView::construct(schema, that)
            .map_err(|e| KVRuntimeError::with_source(KVRuntimeErrorKind::DataMalformed, e))?;

        for _ in 0..column_count {
            let (Some(this_column), Some(that_column)) =
                (this_key.next_column()?, that_key.next_column()?)
            else {
                panic!("primary key comparison error")
            };

            match this_column.0.cmp(this_column.1, that_column.1) {
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

        let mut this_key = PrimaryKeyView::construct(schema, this)
            .map_err(|e| KVRuntimeError::with_source(KVRuntimeErrorKind::DataMalformed, e))?;
        let mut that_key = PrimaryKeyView::construct(schema, that)
            .map_err(|e| KVRuntimeError::with_source(KVRuntimeErrorKind::DataMalformed, e))?;

        for _ in 0..column_count {
            let (Some(this_column), Some(that_column)) =
                (this_key.next_column()?, that_key.next_column()?)
            else {
                panic!("primary key equality error")
            };

            if !this_column.0.eq(this_column.1, that_column.1) {
                return Ok(false);
            }
        }

        Ok(true)
    }
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
