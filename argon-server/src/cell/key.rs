use std::{cmp::Ordering, u64};

use bytes::{BufMut, Bytes, BytesMut};

use super::key_type::KeyType;

const EMPTY_BYTES: Bytes = Bytes::from_static(b"");

/// Stores key part of Cell. Internally data is stored in single bytes array.
/// Data layout is inspired by Cell data structure in HBase/HFile:
/// row_len:        u16
/// row:            [u8]
/// col_family_len: u8
/// col_family:     [u8]
/// col_qualifier:  [u8]
/// timestamp:      u64
/// key_type:       u8
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    key_len: u64,
    data: Bytes,
    row_len: u16,
    col_family_len: u8,
    col_qualifier_len: u64,
    timestamp: u64,
    key_type: KeyType,
}

impl Key {
    pub fn key_len(&self) -> u64 {
        self.key_len
    }

    pub fn bytes(&self) -> Bytes {
        self.data.clone()
    }

    pub fn row(&self) -> &[u8] {
        let start = 2;
        let end = 2 + self.row_len as usize;
        &self.data[start..end]
    }

    pub fn col_family(&self) -> &[u8] {
        let start = 3 + self.row_len as usize;
        let end = start + self.col_family_len as usize;
        &self.data[start..end]
    }

    pub fn col_qualifier(&self) -> &[u8] {
        let start = 3 + self.row_len as usize + self.col_family_len as usize;
        let end = start + self.col_qualifier_len as usize;
        &self.data[start..end]
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn key_type(&self) -> KeyType {
        self.key_type
    }

    pub fn try_from_bytes(data: Bytes) -> Result<Self, String> {
        let key_len: u64 = data
            .len()
            .try_into()
            .map_err(|_| format!("key length cannot be bigger than {}", u64::MAX))?;

        let row_len = u16::from_be_bytes(
            data[0..2]
                .try_into()
                .map_err(|_| "row_len must be 2 bytes")?,
        );

        let col_family_len_start = row_len as usize + 2;
        let col_family_len = u8::from_be_bytes(
            data[col_family_len_start..(col_family_len_start + 1)]
                .try_into()
                .map_err(|_| "col_family_len must be 1 byte")?,
        );

        let col_qualifier_len: u64 =
            key_len - col_family_len_start as u64 - col_family_len as u64 - 10;

        let timestamp_start = key_len as usize - 1 - 8;
        let timestamp: u64 = u64::from_be_bytes(
            data[timestamp_start..(timestamp_start + 8)]
                .try_into()
                .map_err(|_| "timestamp must be 8 bytes")?,
        );

        let key_type_start = key_len as usize - 1;
        let key_type = KeyType::try_from(u8::from_be_bytes(
            data[key_type_start..(key_type_start + 1)]
                .try_into()
                .map_err(|_| "key_type must be 1 byte")?,
        ))?;

        Ok(Self {
            key_len,
            data,
            row_len,
            col_family_len,
            col_qualifier_len,
            timestamp,
            key_type,
        })
    }

    pub fn try_new(
        row: &Bytes,
        col_family: &Bytes,
        col_qualifier: &Bytes,
        timestamp: u64,
        key_type: KeyType,
    ) -> Result<Self, String> {
        let row_len: u16 = row
            .len()
            .try_into()
            .map_err(|_| format!("row length cannot be bigger than {}", u16::MAX))?;

        let col_family_len: u8 = col_family
            .len()
            .try_into()
            .map_err(|_| format!("column family length cannot be bigger than {}", u8::MAX))?;

        let col_qualifier_len: u64 = col_qualifier
            .len()
            .try_into()
            .map_err(|_| format!("key length cannot be bigger than {}", u64::MAX))?;

        let key_len_without_col_qualifier = 12 + row_len as u64 + col_family_len as u64;
        if u64::MAX - key_len_without_col_qualifier < col_qualifier_len {
            return Err(format!("key length cannot be bigger than {}", u64::MAX));
        }

        let key_len = key_len_without_col_qualifier + col_qualifier_len;
        let mut data = BytesMut::with_capacity(key_len as usize);

        data.put_u16(row_len);
        data.put(row.as_ref());
        data.put_u8(col_family_len);
        data.put(col_family.as_ref());
        data.put(col_qualifier.as_ref());
        data.put_u64(timestamp);
        data.put_u8(key_type as u8);

        Ok(Self {
            key_len,
            data: data.freeze(),
            row_len,
            col_family_len,
            col_qualifier_len,
            timestamp,
            key_type,
        })
    }

    pub fn new_first_on_row(row: &Bytes) -> Result<Self, String> {
        Key::try_new(row, &EMPTY_BYTES, &EMPTY_BYTES, u64::MAX, KeyType::Minimum)
    }

    pub fn new_last_on_row(row: &Bytes) -> Result<Self, String> {
        Key::try_new(row, &EMPTY_BYTES, &EMPTY_BYTES, u64::MIN, KeyType::Maximum)
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ord = self.row().cmp(other.row());
        if ord != Ordering::Equal {
            return ord;
        }

        // if self.key_type() == KeyType::Minimum && self
        if self.col_family_len == 0
            && self.col_qualifier_len == 0
            && self.key_type == KeyType::Minimum
        {
            return Ordering::Greater;
        }

        if other.col_family_len == 0
            && other.col_qualifier_len == 0
            && other.key_type() == KeyType::Minimum
        {
            return Ordering::Less;
        }

        let ord = self.col_family().cmp(other.col_family());
        if ord != Ordering::Equal {
            return ord;
        }

        let ord = self.col_qualifier().cmp(other.col_qualifier());
        if ord != Ordering::Equal {
            return ord;
        }

        let ord = other.timestamp.cmp(&self.timestamp);
        if ord != Ordering::Equal {
            return ord;
        }

        let ord = (other.key_type as u8).cmp(&(self.key_type as u8));
        if ord != Ordering::Equal {
            return ord;
        }

        Ordering::Equal
        // other.mvcc_id.cmp(&self.mvcc_id)
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_serialize_then_deserialize() {
        let row = Bytes::from("row_name");
        let cf = Bytes::from("cf");
        let qualifier = Bytes::from("qualifier");
        let timestamp = 0xaa99u64;
        let key_type = KeyType::Put;

        let key = Key::try_new(&row, &cf, &qualifier, timestamp, key_type).unwrap();
        let serialized = key.bytes();
        let key = Key::try_from_bytes(serialized).unwrap();

        assert_eq!(key.row(), row);
        assert_eq!(key.col_family(), cf);
        assert_eq!(key.col_qualifier(), qualifier);
        assert_eq!(key.timestamp(), timestamp);
        assert_eq!(key.key_type(), key_type);
    }

    #[test]
    fn create_empty_serialize_then_deserialize() {
        let row = Bytes::from("");
        let cf = Bytes::from("");
        let qualifier = Bytes::from("");
        let timestamp = 0u64;
        let key_type = KeyType::Minimum;

        let key = Key::try_new(&row, &cf, &qualifier, timestamp, key_type).unwrap();
        let serialized = key.bytes();
        let key = Key::try_from_bytes(serialized).unwrap();

        assert_eq!(key.row(), row);
        assert_eq!(key.col_family(), cf);
        assert_eq!(key.col_qualifier(), qualifier);
        assert_eq!(key.timestamp(), timestamp);
        assert_eq!(key.key_type(), key_type);
    }
}
