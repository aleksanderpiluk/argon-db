use crate::kv::{
    error::KVRuntimeError,
    primary_key::{KVPrimaryKeyComparator, KVPrimaryKeySchema},
};
use bytemuck::{CheckedBitPattern, NoUninit, bytes_of, checked, from_bytes};
use std::{cmp::Ordering, io::Write};

pub trait KVMutation {
    fn timestamp(&self) -> u64;

    fn column_id(&self) -> u16;

    fn mutation_type(&self) -> MutationType;

    fn primary_key_size(&self) -> u16;

    fn value_size(&self) -> u64;

    fn primary_key(&self) -> &[u8];

    fn value(&self) -> &[u8];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, NoUninit, CheckedBitPattern)]
#[repr(u8)]
pub enum MutationType {
    Start = 1,
    Put = 2,
    Delete = 4,
    End = 128,
}

impl MutationType {
    pub fn is_marker(&self) -> bool {
        match *self {
            MutationType::Start => true,
            MutationType::End => true,
            _ => false,
        }
    }
}

impl PartialOrd for MutationType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.clone() as u8).cmp(&(other.clone() as u8)) {
            Ordering::Greater => Some(Ordering::Less),
            Ordering::Equal => Some(Ordering::Equal),
            Ordering::Less => Some(Ordering::Greater),
        }
    }
}

#[derive(Debug)]
pub enum MutationError {
    PrimaryKeySizeExceeded,
    ValueSizeExceeded,
}

pub struct MutationComparator;

impl MutationComparator {
    pub fn cmp<T: KVMutation + ?Sized, U: KVMutation + ?Sized>(
        schema: &KVPrimaryKeySchema,
        this: &T,
        that: &U,
    ) -> Result<Ordering, KVRuntimeError> {
        match KVPrimaryKeyComparator::cmp(schema, this.primary_key(), that.primary_key())? {
            Ordering::Equal => {}
            ord => return Ok(ord),
        }

        match this.timestamp().cmp(&that.timestamp()) {
            Ordering::Equal => {}
            Ordering::Greater => return Ok(Ordering::Less),
            Ordering::Less => return Ok(Ordering::Greater),
        };

        match this.column_id().cmp(&that.column_id()) {
            Ordering::Equal => {}
            ord => return Ok(ord),
        };

        Ok(this.mutation_type().cmp(&that.mutation_type()))
    }

    pub fn eq<T: KVMutation + ?Sized, U: KVMutation + ?Sized>(
        schema: &KVPrimaryKeySchema,
        this: &T,
        that: &U,
    ) -> Result<bool, KVRuntimeError> {
        Ok(
            KVPrimaryKeyComparator::eq(schema, this.primary_key(), that.primary_key())?
                && this.timestamp().eq(&that.timestamp())
                && this.column_id().eq(&that.column_id())
                && this.mutation_type().eq(&that.mutation_type()),
        )
    }
}

#[derive(Debug, Clone)]
pub struct StructuredMutation {
    timestamp: u64,
    column_id: u16,
    mutation_type: MutationType,
    primary_key: Box<[u8]>,
    value: Box<[u8]>,
}

impl StructuredMutation {
    pub fn try_from(
        timestamp: u64,
        column_id: u16,
        mutation_type: MutationType,
        primary_key: Box<[u8]>,
        value: Box<[u8]>,
    ) -> Result<Self, MutationError> {
        if primary_key.len() > u16::MAX as _ {
            return Err(MutationError::PrimaryKeySizeExceeded);
        }

        if value.len() > u64::MAX as _ {
            return Err(MutationError::ValueSizeExceeded);
        }

        Ok(Self {
            timestamp,
            column_id,
            mutation_type,
            primary_key,
            value,
        })
    }

    pub fn from_mutation<T: KVMutation + ?Sized>(mutation: &T) -> Self {
        Self {
            timestamp: mutation.timestamp(),
            column_id: mutation.column_id(),
            mutation_type: mutation.mutation_type(),
            primary_key: mutation.primary_key().to_vec().into_boxed_slice(),
            value: mutation.value().to_vec().into_boxed_slice(),
        }
    }

    pub fn start(primary_key: Box<[u8]>) -> Result<Self, MutationError> {
        Self::try_from(0, 0, MutationType::Start, primary_key, Box::new([]))
    }

    pub fn end(primary_key: Box<[u8]>) -> Result<Self, MutationError> {
        Self::try_from(0, 0, MutationType::End, primary_key, Box::new([]))
    }

    pub fn size(&self) -> usize {
        const CONSTANT_SIZE: usize = 8 + 2 + 1;

        CONSTANT_SIZE + self.primary_key.len() + self.value.len()
    }
}

impl KVMutation for StructuredMutation {
    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn column_id(&self) -> u16 {
        self.column_id
    }

    fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }

    fn primary_key_size(&self) -> u16 {
        self.primary_key.len() as u16
    }

    fn value_size(&self) -> u64 {
        self.value.len() as u64
    }

    fn primary_key(&self) -> &[u8] {
        &self.primary_key
    }

    fn value(&self) -> &[u8] {
        &self.value
    }
}

pub struct SerializedMutationView<'a>(&'a [u8]);

impl<'a> SerializedMutationView<'a> {
    pub fn try_from(data: &'a [u8]) -> Result<Self, ()> {
        let mutation = Self(data);
        let calc_size: usize =
            mutation.value_size() as usize + mutation.primary_key_size() as usize + 22;

        if calc_size == mutation.len() {
            Ok(mutation)
        } else {
            Err(())
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> KVMutation for SerializedMutationView<'a> {
    fn timestamp(&self) -> u64 {
        *from_bytes::<u64>(&self.0[0..8])
    }

    fn column_id(&self) -> u16 {
        *from_bytes::<u16>(&self.0[8..10])
    }

    fn mutation_type(&self) -> MutationType {
        *checked::from_bytes::<MutationType>(&self.0[10..11])
    }

    fn primary_key_size(&self) -> u16 {
        *from_bytes::<u16>(&self.0[11..13])
    }

    fn value_size(&self) -> u64 {
        *from_bytes::<u64>(&self.0[13..21])
    }

    fn primary_key(&self) -> &[u8] {
        let e = 22 + self.primary_key_size() as usize;
        &self.0[22..e]
    }

    fn value(&self) -> &[u8] {
        let s = 22 + self.primary_key_size() as usize;
        let e = s + self.value_size() as usize;
        &self.0[s..e]
    }
}

pub struct MutationUtils;

impl MutationUtils {
    pub fn is_marker(mutation: &impl KVMutation) -> bool {
        mutation.mutation_type().is_marker()
    }

    pub fn as_dyn<T: KVMutation>(mutation: &T) -> &dyn KVMutation {
        mutation
    }
}
