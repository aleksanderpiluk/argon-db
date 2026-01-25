use std::{borrow::Cow, ptr::eq};

use crate::kv::value_type::ValueTypeId;

pub struct Value<'a> {
    type_id: ValueTypeId,
    data: Cow<'a, [u8]>,
}

impl<'a> Value<'a> {
    pub fn new(value_type: ValueTypeId, data: &'a [u8]) -> Result<Self, ()> {
        todo!()
    }

    pub fn eq(&self, other: &Self) -> Result<bool, ()> {
        if self.type_id != other.type_id {
            return Err(());
        }

        todo!()
    }
}

impl Value<'_> {}
