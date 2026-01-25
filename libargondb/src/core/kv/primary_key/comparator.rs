use std::cmp::Ordering;

use crate::kv::{
    core::Comparator,
    primary_key::{PrimaryKeyData, schema::PrimaryKeySchema, view::PrimaryKeyView},
};

struct PrimaryKeyComparator<'a> {
    schema: &'a PrimaryKeySchema,
}

impl<'a> PrimaryKeyComparator<'a> {
    fn new(schema: &'a PrimaryKeySchema) -> Self {
        Self { schema }
    }
}

impl Comparator<(), PrimaryKeyData<'_>> for PrimaryKeyComparator<'_> {
    fn cmp(&self, l: &PrimaryKeyData, r: &PrimaryKeyData) -> Result<std::cmp::Ordering, ()> {
        let mut l_view = PrimaryKeyView::new(self.schema, l);
        let mut r_view = PrimaryKeyView::new(self.schema, r);

        let column_count = self.schema.column_count();
        for _ in 0..column_count {
            let l_column = l_view.next_column()?;
            let r_column = r_view.next_column()?;

            let (Some(l_value), Some(r_value)) = (l_column, r_column) else {
                panic!("primary key comparison error")
            };

            match l_value.0.cmp(l_value.1, r_value.1) {
                Ordering::Equal => {}
                order => return Ok(order),
            }
        }

        let None = l_view.next_column()? else {
            panic!()
        };
        let None = r_view.next_column()? else {
            panic!()
        };

        Ok(Ordering::Equal)
    }

    fn eq(&self, this: &PrimaryKeyData, that: &PrimaryKeyData) -> Result<bool, ()> {
        let mut l_view = PrimaryKeyView::new(self.schema, l);
        let mut r_view = PrimaryKeyView::new(self.schema, r);

        let column_count = self.schema.column_count();
        for _ in 0..column_count {
            let l_column = l_view.next_column()?;
            let r_column = r_view.next_column()?;

            let (Some(l_value), Some(r_value)) = (l_column, r_column) else {
                panic!("primary key comparison error")
            };

            if !l_value.0.eq(l_value.1, r_value.1) {
                return Ok(false);
            }
        }

        let None = l_view.next_column()? else {
            panic!()
        };
        let None = r_view.next_column()? else {
            panic!()
        };

        Ok(true)
    }
}
