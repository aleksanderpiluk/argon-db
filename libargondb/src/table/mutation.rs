use std::cmp;

use bytemuck::{cast_ref, from_bytes};

use crate::PartitionKey;

#[derive(Eq, Ord)]
struct Mutation<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for Mutation<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl Mutation<'_> {
    fn timestamp(&self) -> &u64 {
        from_bytes::<u64>(&self.0[0..8])
    }

    fn column_id(&self) -> &u16 {
        from_bytes::<u16>(&self.0[8..10])
    }

    fn mutation_type(&self) -> &MutationType {
        // from_bytes::<MutationType>(&self.0[10..11])
        todo!()
    }

    fn partition_key_size(&self) -> &u16 {
        from_bytes::<u16>(&self.0[11..13])
    }

    fn value_end(&self) -> &u64 {
        from_bytes::<u64>(&self.0[13..21])
    }

    fn partition_key(&self) -> &PartitionKey {
        todo!()
    }
}

#[derive(PartialEq, Eq, Ord, Clone, Copy)]
#[repr(u8)]
enum MutationType {
    Put = 1,
    Delete = 2,
}

impl PartialOrd for MutationType {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self.clone() as u8).cmp(&(other.clone() as u8)) {
            cmp::Ordering::Equal => Some(cmp::Ordering::Equal),
            cmp::Ordering::Greater => Some(cmp::Ordering::Less),
            cmp::Ordering::Less => Some(cmp::Ordering::Greater),
        }
    }
}

impl PartialEq for Mutation<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.mutation_type() == other.mutation_type()
            && self.column_id() == other.column_id()
            && self.timestamp() == other.timestamp()
            && self.partition_key_size() == other.partition_key_size()
            && self.partition_key() == other.partition_key()
    }
}

impl PartialOrd for Mutation<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.partition_key().cmp(other.partition_key()) {
            cmp::Ordering::Equal => {}
            ord => return Some(ord),
        };

        match self.column_id().cmp(other.column_id()) {
            cmp::Ordering::Equal => {}
            ord => return Some(ord),
        };

        match self.timestamp().cmp(other.timestamp()) {
            cmp::Ordering::Equal => {}
            cmp::Ordering::Greater => return Some(cmp::Ordering::Less),
            cmp::Ordering::Less => return Some(cmp::Ordering::Greater),
        };

        Some(self.mutation_type().cmp(other.mutation_type()))
    }
}
