mod merge_iter;
mod print_iter;
mod shadowing_iter;

pub use print_iter::PrintIter;
pub use shadowing_iter::ShadowingIter;

// pub trait ScanIterator {
//     async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>>;
//     fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>>;
// }

pub trait MutationsIterator<'a, Item> {
    async fn next_mutation(&mut self) -> Option<Item>;
    fn peek_mutation(&self) -> Option<&Item>;
}
