type HeapItem<'a> = Box<dyn super::MutationsIterator<'a, ()>>;

struct MergeIter<'a> {
    heap: binary_heap_plus::BinaryHeap<HeapItem<'a>>,
}

// type BoxKVScanIterator = Box<dyn KVScanIterator + Send + Sync>;

// pub struct KVMergeScanIter {
//     heap: BinaryHeap<BoxKVScanIterator, KVMergeScanIterComparator>,
// }

// impl KVMergeScanIter {
//     pub fn new(schema: KVPrimaryKeySchema) -> Self {
//         Self {
//             heap: BinaryHeap::from_vec_cmp(vec![], KVMergeScanIterComparator { schema }),
//         }
//     }

//     pub fn add_iter(&mut self, iter: BoxKVScanIterator) {
//         self.heap.push(iter);
//     }
// }

// #[async_trait]
// impl KVScanIterator for KVMergeScanIter {
//     async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
//         let Some(mut iter) = self.heap.peek_mut() else {
//             return None;
//         };

//         let mutation = iter.next_mutation().await;

//         mutation
//     }

//     fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>> {
//         match self.heap.peek() {
//             Some(iter) => iter.peek_mutation(),
//             None => None,
//         }
//     }
// }

// struct KVMergeScanIterComparator {
//     schema: KVPrimaryKeySchema,
// }

// impl Compare<BoxKVScanIterator> for KVMergeScanIterComparator {
//     fn compare(&self, l: &BoxKVScanIterator, r: &BoxKVScanIterator) -> Ordering {
//         match (l.peek_mutation(), r.peek_mutation()) {
//             (None, None) => Ordering::Equal,
//             (Some(_), None) => Ordering::Greater,
//             (None, Some(_)) => Ordering::Less,
//             (Some(a), Some(b)) => MutationComparator::cmp(&self.schema, a.mutation(), b.mutation())
//                 .unwrap()
//                 .reverse(),
//         }
//     }
// }
