use async_trait::async_trait;

use crate::kv::{KVScanIterator, KVScanIteratorItem, KVTableSchema, mutation::MutationUtils};

pub struct PrintIter<T: KVScanIterator + Send + Sync + 'static> {
    ctx: String,
    inner: T,
    schema: KVTableSchema,
}

impl<T: KVScanIterator + Send + Sync + 'static> PrintIter<T> {
    pub fn new(ctx: impl AsRef<str>, inner: T, schema: KVTableSchema) -> Self {
        Self {
            ctx: ctx.as_ref().to_string(),
            inner,
            schema,
        }
    }
}

#[async_trait]
impl<T: KVScanIterator + Send + Sync + 'static> KVScanIterator for PrintIter<T> {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        let item = self.inner.next_mutation().await;

        #[cfg(debug_assertions)]
        println!(
            "[PrintIter/{}] next mutation {}",
            self.ctx,
            match &item {
                Some(mutation) =>
                    MutationUtils::debug_fmt(&self.schema, mutation.mutation()).unwrap(),
                None => "None".to_string(),
            }
        );

        item
    }

    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>> {
        self.inner.peek_mutation()
    }
}
