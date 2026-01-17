use std::{
    cmp::Ordering,
    mem::{replace, take},
};

use async_trait::async_trait;

use crate::kv::{
    KVRow, KVRuntimeError, KVScanIterator, KVScanIteratorItem, KVTableSchema,
    mutation::{MutationComparator, MutationUtils},
    primary_key::KVPrimaryKeySchema,
    row::KVRowBuilder,
};

type BoxKVScanIterator = Box<dyn KVScanIterator + Send + Sync>;

pub struct KVMergeScanIter {
    table_schema: KVTableSchema,
    schema: KVPrimaryKeySchema,
    heap: Vec<BoxKVScanIterator>,
}

impl KVMergeScanIter {
    pub fn new(table_schema: KVTableSchema, schema: KVPrimaryKeySchema) -> Self {
        Self {
            table_schema,
            schema,
            heap: vec![],
        }
    }

    pub fn add_iter(&mut self, iter: BoxKVScanIterator) {
        self.heap.push(iter);
        self.heapify();
    }

    fn heapify(&mut self) {
        self.heap
            .sort_by(|a, b| match (a.peek_mutation(), b.peek_mutation()) {
                (None, None) => Ordering::Equal,
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (Some(a), Some(b)) => {
                    MutationComparator::cmp(&self.schema, a.mutation(), b.mutation()).unwrap()
                }
            });

        let mut out = String::from("");
        for iter in &self.heap {
            out += &format!(
                "{}, ",
                match &iter.peek_mutation() {
                    Some(mutation) =>
                        MutationUtils::debug_fmt(&self.table_schema, mutation.mutation()).unwrap(),
                    None => "None".to_string(),
                }
            );
        }
        #[cfg(debug_assertions)]
        println!("Heapified: {}", out);
    }
}

#[async_trait]
impl KVScanIterator for KVMergeScanIter {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        let Some(iter) = self.heap.get_mut(0) else {
            return None;
        };

        let mutation = iter.next_mutation().await;

        self.heapify();

        mutation
    }

    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>> {
        match self.heap.get(0) {
            Some(ref iter) => iter.peek_mutation(),
            None => None,
        }
    }
}

pub struct KVRowIter {
    table_schema: KVTableSchema,
    iter: BoxKVScanIterator,
    current_row: Option<KVRowBuilder>,
    finished: bool,
}

impl KVRowIter {
    pub fn new(table_schema: KVTableSchema, iter: BoxKVScanIterator) -> Self {
        Self {
            table_schema,
            iter,
            current_row: None,
            finished: false,
        }
    }

    pub async fn next_row(&mut self) -> Result<Option<KVRow>, KVRuntimeError> {
        if self.finished {
            return Ok(None);
        }

        loop {
            let mutation = self.iter.next_mutation().await;

            match mutation {
                Some(item) => match self.current_row.as_mut() {
                    Some(row) => {
                        if row.can_add(&item)? {
                            row.add(item)?;
                        } else {
                            let row = replace(
                                &mut self.current_row,
                                Some(KVRowBuilder::new(self.table_schema.clone(), item)),
                            );

                            return Ok(row.map(|r| r.into()));
                        }
                    }
                    None => {
                        self.current_row = Some(KVRowBuilder::new(self.table_schema.clone(), item));
                    }
                },
                None => {
                    self.finished = true;
                    let row = take(&mut self.current_row);

                    return Ok(row.map(|r| r.into()));
                }
            }
        }
    }
}
