use binary_heap_plus::BinaryHeap;
use compare::Compare;
use std::{
    cmp::Ordering,
    mem::{replace, take},
};

use async_trait::async_trait;

use crate::kv::{
    KVRow, KVRuntimeError, KVScanIterator, KVScanIteratorItem, KVTableSchema,
    mutation::MutationComparator, primary_key::KVPrimaryKeySchema, row::KVRowBuilder,
};

type BoxKVScanIterator = Box<dyn KVScanIterator + Send + Sync>;

pub struct KVMergeScanIter {
    heap: BinaryHeap<BoxKVScanIterator, KVMergeScanIterComparator>,
}

impl KVMergeScanIter {
    pub fn new(schema: KVPrimaryKeySchema) -> Self {
        Self {
            heap: BinaryHeap::from_vec_cmp(vec![], KVMergeScanIterComparator { schema }),
        }
    }

    pub fn add_iter(&mut self, iter: BoxKVScanIterator) {
        self.heap.push(iter);
    }
}

#[async_trait]
impl KVScanIterator for KVMergeScanIter {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        let Some(mut iter) = self.heap.peek_mut() else {
            return None;
        };

        let mutation = iter.next_mutation().await;

        mutation
    }

    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>> {
        match self.heap.peek() {
            Some(iter) => iter.peek_mutation(),
            None => None,
        }
    }
}

struct KVMergeScanIterComparator {
    schema: KVPrimaryKeySchema,
}

impl Compare<BoxKVScanIterator> for KVMergeScanIterComparator {
    fn compare(&self, l: &BoxKVScanIterator, r: &BoxKVScanIterator) -> Ordering {
        match (l.peek_mutation(), r.peek_mutation()) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (Some(a), Some(b)) => MutationComparator::cmp(&self.schema, a.mutation(), b.mutation())
                .unwrap()
                .reverse(),
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
