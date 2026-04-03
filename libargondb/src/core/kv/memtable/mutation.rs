// #[derive(Debug)]
// struct MemtableMutation {
//     primary_key_schema: Arc<KVPrimaryKeySchema>,
//     inner: StructuredMutation,
// }

// impl MemtableMutation {
//     fn start(primary_key_schema: Arc<KVPrimaryKeySchema>, primary_key: Box<[u8]>) -> Self {
//         Self {
//             primary_key_schema,
//             mutation: StructuredMutation::start(primary_key).unwrap(),
//         }
//     }

//     fn end(primary_key_schema: Arc<KVPrimaryKeySchema>, primary_key: Box<[u8]>) -> Self {
//         Self {
//             primary_key_schema,
//             mutation: StructuredMutation::end(primary_key).unwrap(),
//         }
//     }

//     fn as_mutation(&self) -> &dyn KVMutation {
//         &self.mutation
//     }
// }

// impl Eq for MemtableMutation {}

// impl PartialEq for MemtableMutation {
//     fn eq(&self, other: &Self) -> bool {
//         assert!(Arc::ptr_eq(
//             &self.primary_key_schema,
//             &other.primary_key_schema
//         ));

//         MutationComparator::eq(&self.primary_key_schema, &self.mutation, &other.mutation).unwrap()
//     }
// }

// impl Ord for MemtableMutation {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         assert!(Arc::ptr_eq(
//             &self.primary_key_schema,
//             &other.primary_key_schema
//         ));

//         MutationComparator::cmp(&self.primary_key_schema, &self.mutation, &other.mutation).unwrap()
//     }
// }

// impl PartialOrd for MemtableMutation {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }
