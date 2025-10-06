use std::cmp::Ordering;

use bytemuck::from_bytes;

use crate::kv::column_type::{self, ColumnType};

pub struct PrimaryKeySchema(Box<[u8]>);

impl PrimaryKeySchema {
    fn column_count(&self) -> u8 {
        self.0[0]
    }

    fn column_type(&self, idx: usize) -> impl ColumnType {
        todo!();
        column_type::Bytes
    }
}

pub struct PrimaryKeyView<'a> {
    schema: &'a PrimaryKeySchema,
    key: &'a [u8],

    column_count: u8,
    /** Size of all column length values */
    data_shift: usize,

    column_idx: usize,
    value_ptr: usize,
}

impl<'a> PrimaryKeyView<'a> {
    fn new(schema: &'a PrimaryKeySchema, key: &'a [u8]) -> Self {
        let column_count = schema.column_count();
        let data_shift = 2 * column_count as usize;

        let column_idx = 0;
        let value_ptr = data_shift;

        Self {
            schema,
            key,
            column_count,
            data_shift,
            column_idx,
            value_ptr,
        }
    }

    fn next_column(&mut self) -> Option<(impl ColumnType, &[u8])> {
        let column_idx = self.column_idx;
        let value_ptr = self.value_ptr;

        if column_idx >= self.column_count as usize {
            return None;
        }

        let size_ptr = 2 * column_idx;
        let value_size = *from_bytes::<u16>(&self.key[size_ptr..(size_ptr + 2)]) as usize;

        let col_type = self.schema.column_type(column_idx);
        let col_value = &self.key[value_ptr..(value_ptr + value_size)];

        self.value_ptr = value_ptr + value_size;
        self.column_idx = column_idx + 1;

        Some((col_type, col_value))
    }
}

pub struct PrimaryKeyComparator;

impl PrimaryKeyComparator {
    pub fn cmp(schema: &PrimaryKeySchema, this: &[u8], that: &[u8]) -> Ordering {
        let column_count = schema.column_count();

        let mut this_key = PrimaryKeyView::new(schema, this);
        let mut that_key = PrimaryKeyView::new(schema, that);

        for _ in 0..column_count {
            let Some(this_column) = this_key.next_column() else {
                panic!("primary key comparison error")
            };
            let Some(that_column) = that_key.next_column() else {
                panic!("primary key comparison error")
            };

            match Self::column_cmp(this_column.0, this_column.1, that_column.1) {
                Ordering::Equal => {}
                order => return order,
            }
        }

        Ordering::Equal
    }

    pub fn eq(schema: &PrimaryKeySchema, this: &[u8], that: &[u8]) -> bool {
        let column_count = schema.column_count();

        let mut this_key = PrimaryKeyView::new(schema, this);
        let mut that_key = PrimaryKeyView::new(schema, that);

        for _ in 0..column_count {
            let Some(this_column) = this_key.next_column() else {
                panic!("primary key equality error")
            };
            let Some(that_column) = that_key.next_column() else {
                panic!("primary key equality error")
            };

            if !Self::column_eq(this_column.0, this_column.1, that_column.1) {
                return false;
            }
        }

        true
    }

    fn column_cmp<T: ColumnType>(_col_type: T, this: &[u8], that: &[u8]) -> Ordering {
        T::cmp(this, that)
    }

    fn column_eq<T: ColumnType>(_col_type: T, this: &[u8], that: &[u8]) -> bool {
        T::eq(this, that)
    }
}
