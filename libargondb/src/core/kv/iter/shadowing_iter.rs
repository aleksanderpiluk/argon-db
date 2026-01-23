use std::collections::BTreeSet;

use async_trait::async_trait;

use crate::kv::{
    KVScanIterator, KVScanIteratorItem,
    primary_key::{KVPrimaryKeyComparator, KVPrimaryKeySchema},
};

pub struct ShadowingIter<T: KVScanIterator + Send + Sync + 'static> {
    current_mask: Option<(Vec<u8>, BTreeSet<u16>)>,
    current_mutation: Option<Box<dyn KVScanIteratorItem + Send + Sync + 'static>>,
    finished: bool,
    inner: T,
    schema: KVPrimaryKeySchema,
}

impl<T: KVScanIterator + Send + Sync + 'static> ShadowingIter<T> {
    pub async fn new(inner: T, schema: KVPrimaryKeySchema) -> Self {
        let mut this = Self {
            current_mask: None,
            current_mutation: None,
            finished: false,
            inner,
            schema,
        };

        this.fetch_next_mutation().await;

        this
    }

    async fn fetch_next_mutation(&mut self) {
        while !self.finished {
            let next_mutation = self.inner.next_mutation().await;

            let Some(item) = next_mutation else {
                self.finished = true;
                self.current_mutation = None;
                break;
            };

            self.replace_mask_if_necessary(&item);

            if !self.should_be_skipped(&item) {
                self.add_to_mask(&item);
                self.current_mutation = Some(item);
                break;
            }
        }
    }

    fn should_be_skipped(
        &self,
        item: &Box<dyn KVScanIteratorItem + Send + Sync + 'static>,
    ) -> bool {
        let Some((_, columns_set)) = &self.current_mask else {
            unreachable!()
        };

        columns_set.contains(&item.mutation().column_id())
    }

    fn add_to_mask(&mut self, item: &Box<dyn KVScanIteratorItem + Send + Sync + 'static>) {
        let Some((_, columns_set)) = &mut self.current_mask else {
            unreachable!()
        };

        columns_set.insert(item.mutation().column_id());
    }

    fn replace_mask_if_necessary(
        &mut self,
        mutation: &Box<dyn KVScanIteratorItem + Send + Sync + 'static>,
    ) {
        let Some((mask_pk, ..)) = &self.current_mask else {
            self.new_empty_mask(mutation);
            return;
        };

        if !KVPrimaryKeyComparator::eq(&self.schema, mask_pk, mutation.primary_key()).unwrap() {
            self.new_empty_mask(mutation);
            return;
        }
    }

    fn new_empty_mask(&mut self, item: &Box<dyn KVScanIteratorItem + Send + Sync + 'static>) {
        let mask_pk = item.primary_key().to_vec();

        self.current_mask = Some((mask_pk, BTreeSet::new()));
    }
}

#[async_trait]
impl<T: KVScanIterator + Send + Sync + 'static> KVScanIterator for ShadowingIter<T> {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        let item = std::mem::take(&mut self.current_mutation);

        self.fetch_next_mutation().await;

        item
    }

    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>> {
        self.current_mutation.as_ref()
    }
}
