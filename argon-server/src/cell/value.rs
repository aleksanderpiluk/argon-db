use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct Value {
    value_len: u64,
    data: Bytes,
}

impl Value {
    pub fn bytes(&self) -> Bytes {
        self.data.clone()
    }

    pub fn value_len(&self) -> u64 {
        self.value_len
    }

    pub fn try_from_bytes(value: &Bytes) -> Result<Self, String> {
        let value_len: u64 = value
            .len()
            .try_into()
            .map_err(|_| format!("value length cannot be bigger than {}", u64::MAX))?;

        Ok(Self {
            value_len,
            data: value.clone(),
        })
    }
}
