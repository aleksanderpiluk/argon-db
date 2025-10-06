use std::io::Write;

use bytemuck::{bytes_of, checked, from_bytes};

use crate::{
    data_types::MutationType,
    traits::{Mutation, cmp_mutations, eq_mutations},
};

#[derive(Eq, Ord)]
pub struct InMemoryMutation(Box<[u8]>);

impl InMemoryMutation {
    fn from(
        timestamp: u64,
        mutation_type: MutationType,
        column_id: u16,
        partition_key: &[u8],
        value: &[u8],
    ) -> Self {
        let size = 8 + 2 + 2 + 2 + 8 + partition_key.len() + value.len();
        let mut vec = Vec::with_capacity(size);
        vec.extend_from_slice(bytes_of(&timestamp));
        vec.extend_from_slice(bytes_of(&column_id));
        vec.extend_from_slice(bytes_of(&mutation_type));
        vec.extend_from_slice(bytes_of(&(partition_key.len() as u16)));
        vec.extend_from_slice(bytes_of(&(value.len() as u64)));
        vec.extend_from_slice(partition_key);
        vec.extend_from_slice(value);

        Self(vec.into_boxed_slice())
    }

    pub fn from_mutation(mutation: impl Mutation) -> Self {
        todo!()
    }

    fn as_view(&self) -> InMemoryMutationView<'_> {
        InMemoryMutationView(&self.0)
    }
}

impl Mutation for InMemoryMutation {
    fn timestamp(&self) -> &u64 {
        todo!()
    }

    fn column_id(&self) -> &u16 {
        todo!()
    }

    fn mutation_type(&self) -> &MutationType {
        todo!()
    }

    fn primary_key_size(&self) -> &u16 {
        todo!()
    }

    fn value_size(&self) -> &u64 {
        todo!()
    }

    fn primary_key(&self) -> &[u8] {
        todo!()
    }

    fn value(&self) -> &[u8] {
        todo!()
    }
}

impl<T: Mutation> PartialEq<T> for InMemoryMutation {
    fn eq(&self, other: &T) -> bool {
        self.as_view().eq(other)
    }
}

impl<T: Mutation> PartialOrd<T> for InMemoryMutation {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.as_view().partial_cmp(other)
    }
}

#[derive(Eq, Ord)]
pub struct InMemoryMutationView<'a>(&'a [u8]);

impl<'a> InMemoryMutationView<'a> {
    pub fn try_from(data: &'a [u8]) -> Result<Self, ()> {
        let mutation = Self(data);
        let calc_size: usize =
            *mutation.value_size() as usize + *mutation.primary_key_size() as usize + 22;

        if calc_size == mutation.len() {
            Ok(mutation)
        } else {
            Err(())
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn write<W: Write>(writer: &mut W, mutation: impl Mutation) {
        writer.write(bytes_of(mutation.timestamp()));
        writer.write(bytes_of(mutation.column_id()));
        writer.write(bytes_of(mutation.mutation_type()));
        writer.write(bytes_of(mutation.primary_key_size()));
        writer.write(bytes_of(mutation.value_size()));
        writer.write(mutation.primary_key());
        writer.write(mutation.value());
    }
}

impl<'a> Mutation for InMemoryMutationView<'a> {
    fn timestamp(&self) -> &u64 {
        from_bytes::<u64>(&self.0[0..8])
    }

    fn column_id(&self) -> &u16 {
        from_bytes::<u16>(&self.0[8..10])
    }

    fn mutation_type(&self) -> &MutationType {
        checked::from_bytes::<MutationType>(&self.0[10..11])
    }

    fn primary_key_size(&self) -> &u16 {
        from_bytes::<u16>(&self.0[11..13])
    }

    fn value_size(&self) -> &u64 {
        from_bytes::<u64>(&self.0[13..21])
    }

    fn primary_key(&self) -> &[u8] {
        let e = 22 + *self.primary_key_size() as usize;
        &self.0[22..e]
    }

    fn value(&self) -> &[u8] {
        let s = 22 + *self.primary_key_size() as usize;
        let e = s + *self.value_size() as usize;
        &self.0[s..e]
    }
}

impl<T: Mutation> PartialEq<T> for InMemoryMutationView<'_> {
    fn eq(&self, other: &T) -> bool {
        eq_mutations(self, other)
    }
}

impl<T: Mutation> PartialOrd<T> for InMemoryMutationView<'_> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        Some(cmp_mutations(self, other))
    }
}
