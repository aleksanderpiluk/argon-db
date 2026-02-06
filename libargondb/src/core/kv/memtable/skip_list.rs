use super::super::mutation;
use super::super::primary_key;
use super::super::scannable;
use std::sync::Arc;

pub struct SkipList {
    schema: Arc<primary_key::Schema>,
    inner: crossbeam_skiplist::SkipSet<SkipListMutation>,
}

struct SkipListMutation {
    schema: Arc<primary_key::Schema>,
    marker: scannable::RangeScanMarker,
    inner: super::MemtableMutation,
}

impl std::cmp::Eq for SkipListMutation {}

impl std::cmp::PartialEq for SkipListMutation {
    fn eq(&self, other: &Self) -> bool {
        assert!(Arc::ptr_eq(&self.schema, &other.schema));

        let comparator = mutation::MutationComparator::new(&self.schema);

        comparator
            .eq(&self.primary_key_schema, &self.mutation, &other.mutation)
            .unwrap()
    }
}

impl std::cmp::PartialOrd for SkipListMutation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for SkipListMutation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        assert!(Arc::ptr_eq(
            &self.primary_key_schema,
            &other.primary_key_schema
        ));

        let comparator = mutation::MutationComparator::new(&self.schema);

        comparator
            .cmp(&self.primary_key_schema, &self.mutation, &other.mutation)
            .unwrap()
    }
}

impl std::borrow::Borrow<scannable::RangeScanMarker> for SkipListMutation {
    fn borrow(&self) -> &scannable::RangeScanMarker {
        &self.marker
    }
}

pub struct ScanIter<'a> {}

impl ScanIter<'a> {
    pub fn for_range_scan(skip_list: &'a SkipList, params: scannable::RangeScanParams) -> Self {
        todo!()
    }

    pub fn for_row_scan(skip_list: &'a SkipList, params: scannable::RowScanParams) -> Self {
        todo!()
    }
}

//     fn get_range_iterator<'a>(
//         &'a self,
//         scan: &KVRangeScan,
//     ) -> Result<MemtableScanResultsIterInner<'a>, KVRuntimeError> {
//         let from = scan.from();
//         let to = scan.to();

//         match (from, to) {
//             (KVPrimaryKeyMarker::Start, KVPrimaryKeyMarker::End) => {
//                 Ok(Box::new(self.inner.range(..).into_iter()))
//             }
//             (KVPrimaryKeyMarker::Start, KVPrimaryKeyMarker::Key(pk)) => Ok(Box::new(
//                 self.inner
//                     .range(..MemtableMutation::end(self.primary_key_schema.clone(), pk.clone()))
//                     .into_iter(),
//             )),
//             (KVPrimaryKeyMarker::Key(pk), KVPrimaryKeyMarker::End) => Ok(Box::new(
//                 self.inner
//                     .range(MemtableMutation::start(self.primary_key_schema.clone(), pk.clone())..)
//                     .into_iter(),
//             )),
//             (KVPrimaryKeyMarker::Key(pk_from), KVPrimaryKeyMarker::Key(pk_to)) => Ok(Box::new(
//                 self.inner
//                     .range(
//                         MemtableMutation::start(self.primary_key_schema.clone(), pk_from.clone())
//                             ..MemtableMutation::end(self.primary_key_schema.clone(), pk_to.clone()),
//                     )
//                     .into_iter(),
//             )),
//             _ => {
//                 return Err(KVRuntimeError::with_msg(
//                     KVRuntimeErrorKind::DataMalformed,
//                     "invalid scan range",
//                 ));
//             }
//         }
//     }
