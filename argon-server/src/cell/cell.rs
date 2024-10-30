use std::{io::Write, result};

use super::key::Key;
use super::value::Value;
use bytes::{BufMut, Bytes, BytesMut};

/// Cell is structure storing key and value.
/// Internally data is stored in single bytes array.
/// Data layout is inspired by Cell data structure in HBase/HFile.
/// key_len:    u64
/// value_len:  u64
/// key:        [u8]
/// value:      [u8]
#[derive(Debug, Clone)]
pub struct Cell {
    pub key: Key,
    pub value: Value,
}

impl Cell {
    /// WARNING: THIS FUNCTION CREATES NEW BYTES INSTANCE IN EVERY CALL
    pub fn bytes(&self) -> Bytes {
        let mut out = BytesMut::new();

        out.put_u64(self.key.key_len());
        out.put_u64(self.value.value_len());
        out.put(self.key.bytes());
        out.put(self.value.bytes());

        out.freeze()
    }

    pub fn write_to_writer<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.write_all(&u64::to_be_bytes(self.key.key_len()))?;
        w.write_all(&u64::to_be_bytes(self.value.value_len()))?;
        w.write_all(&self.key.bytes())?;
        w.write_all(&self.value.bytes())?;
        Ok(())
    }

    pub fn try_from_bytes(data: Bytes) -> Result<Self, String> {
        let key_len = u64::from_be_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| "key_len must be 8 bytes")?,
        );

        let value_len = u64::from_be_bytes(
            data[8..16]
                .try_into()
                .map_err(|_| "value_len must be 8 bytes")?,
        );

        let key_start = 16;
        let key_end = 16 + key_len as usize;
        let key_bytes = data.slice(key_start..key_end);
        let key = Key::try_from_bytes(key_bytes)?;

        let value_start = key_end;
        let value_end = key_end + value_len as usize;
        let value_bytes = data.slice(value_start..value_end);
        let value = Value::try_from_bytes(&value_bytes)?;

        Ok(Self { key, value })
    }

    pub fn new(key: Key, value: Value) -> Self {
        Self { key, value }
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl Eq for Cell {}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::super::key_type::KeyType;

    use super::*;

    #[test]
    fn new_cell_serialize_then_deserialize() {
        let row = Bytes::from("row_name");
        let cf = Bytes::from("cf");
        let qualifier = Bytes::from("qualifier");
        let timestamp = 0xaa99u64;
        let key_type = KeyType::Put;

        let key = Key::try_new(&row, &cf, &qualifier, timestamp, key_type).unwrap();

        let value_bytes = Bytes::from("lorem ipsum dolor sit amet...");
        let value = Value::try_from_bytes(&value_bytes).unwrap();

        let cell = Cell::new(key, value);
        let serialized = cell.bytes();
        let cell = Cell::try_from_bytes(serialized).unwrap();

        assert_eq!(cell.key.row(), row);
        assert_eq!(cell.key.col_family(), cf);
        assert_eq!(cell.key.col_qualifier(), qualifier);
        assert_eq!(cell.key.timestamp(), timestamp);
        assert_eq!(cell.key.key_type(), key_type);

        assert_eq!(cell.value.bytes(), value_bytes);
    }

    #[test]
    fn new_cell_serialize_to_writer() {
        let row = Bytes::from("row_name");
        let cf = Bytes::from("cf");
        let qualifier = Bytes::from("qualifier");
        let timestamp = 0xaa99u64;
        let key_type = KeyType::Put;

        let key = Key::try_new(&row, &cf, &qualifier, timestamp, key_type).unwrap();

        let value_bytes = Bytes::from("lorem ipsum dolor sit amet...");
        let value = Value::try_from_bytes(&value_bytes).unwrap();

        let cell = Cell::new(key, value);

        let mut writer = Cursor::new(Vec::new());

        cell.write_to_writer(&mut writer).unwrap();
        writer.flush().unwrap();

        assert_eq!(cell.bytes(), writer.get_ref());
    }
}
