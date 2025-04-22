use std::{cmp::Ordering, ops::Deref};

// use byteorder::{BigEndian, ByteOrder};
use bytes::Bytes;

// /// Cell data layout
// ///[ cell_length (u64), key_length (u16), column_length (u16), value_len (u32), timestamp (u64), key ([u8]), column ([u8]), value ([u8]) ]
// #[derive(Clone)]
// pub struct Cell(Bytes);

// impl Cell {
//     fn bytes(&self) -> Bytes {
//         self.0.clone()
//     }

//     fn column(&self) -> Option<&[u8]> {
//         let column_len = self.column_len();
//         if column_len == 0 {
//             return None;
//         }

//         let Self(data) = self;
//         let key_len = self.key_len() as usize;

//         let s = 24 + key_len;
//         let e = s + column_len as usize;

//         Some(&data[s..e])
//     }

//     fn key(&self) -> Option<&[u8]> {
//         let key_len = self.key_len();
//         if key_len == 0 {
//             return None;
//         }

//         let Self(data) = self;
//         Some(&data[24..(24 + key_len as usize)])
//     }

//     fn timestamp(&self) -> u64 {
//         let Self(data) = self;
//         BigEndian::read_u64(&data[16..24])
//     }

//     fn value(&self) -> Option<&[u8]> {
//         let value_len = self.value_len();
//         if value_len == 0 {
//             return None;
//         }

//         let Self(data) = self;
//         let e = self.len() as usize;
//         let s = e - value_len as usize;
//         Some(&data[s..e])
//     }

//     fn key_len(&self) -> u16 {
//         let Self(data) = self;
//         BigEndian::read_u16(&data[8..10])
//     }

//     fn column_len(&self) -> u16 {
//         let Self(data) = self;
//         BigEndian::read_u16(&data[10..12])
//     }

//     fn value_len(&self) -> u32 {
//         let Self(data) = self;
//         BigEndian::read_u32(&data[12..16])
//     }

//     fn len(&self) -> u64 {
//         let Self(data) = self;
//         BigEndian::read_u64(&data[0..8])
//     }

//     fn key_begin(key: Bytes) {}

//     fn key_end(key: Bytes) {}

//     fn insert(key: Bytes, column: Bytes, timestamp: u64, value: Bytes) {}

//     fn delete(key: Bytes, column: Bytes, timestamp: u64) {}

//     fn column_delete(column: Bytes, timestamp: u64) {}

//     fn new(
//         key: Option<Bytes>,
//         column: Option<Bytes>,
//         timestamp: u64,
//         value: Option<Bytes>,
//     ) -> Self {
//         let key_len = key
//             .as_ref()
//             .map_or(0u16, |b| u16::try_from(b.len()).unwrap());
//         let column_len = column
//             .as_ref()
//             .map_or(0u16, |b| u16::try_from(b.len()).unwrap());
//         let value_len: u32 = value
//             .as_ref()
//             .map_or(0u32, |b| u32::try_from(b.len()).unwrap());

//         let capacity = 24 + key_len as u64 + column_len as u64 + value_len as u64;
//         let mut buf = BytesMut::with_capacity(capacity as usize);

//         buf.put_u64(capacity);

//         buf.put_u16(key_len);
//         buf.put_u16(column_len);
//         buf.put_u32(value_len);

//         buf.put_u64(timestamp);

//         if let Some(key) = key {
//             buf.put(key);
//         }

//         if let Some(column) = column {
//             buf.put(column);
//         }

//         if let Some(value) = value {
//             buf.put(value);
//         }

//         Self(buf.freeze())
//     }
// }

// impl From<Bytes> for Cell {
//     fn from(value: Bytes) -> Self {
//         Self(value)
//     }
// }

// impl Ord for Cell {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         match (self.key(), other.key()) {
//             (None, None)
//         }
//     }
// }

#[derive(Debug)]
pub struct Cell(Bytes);

impl Deref for Cell {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Cell {
    pub fn cell_index(&self) -> CellIndex {
        todo!()
    }

    pub fn value(&self) -> Value {
        todo!()
    }
}

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cell_index().cmp(&other.cell_index())
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Cell {}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.cell_index().eq(&other.cell_index())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CellIndex(Bytes);

impl CellIndex {
    pub fn cell_type(&self) -> CellType {
        todo!()
    }

    pub fn key(&self) -> Key {
        todo!()
    }

    pub fn column_name(&self) -> ColumnName {
        todo!()
    }

    pub fn timestamp(&self) -> Timestamp {
        todo!()
    }
}

impl Ord for CellIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.eq(other) {
            return Ordering::Equal;
        }

        if self.key().is_empty() && self.cell_type() == CellType::Minimum {
            return Ordering::Less;
        }

        if self.key().is_empty() && self.cell_type() == CellType::Maximum {
            return Ordering::Greater;
        }

        let ord = self.key().cmp(&other.key());
        if ord != Ordering::Equal {
            return ord;
        }

        let ord = self.column_name().cmp(&other.column_name());
        if ord != Ordering::Equal {
            return ord;
        }

        let ord = other.timestamp().cmp(&self.timestamp());
        if ord != Ordering::Equal {
            return ord;
        }

        other.cell_type().cmp(&self.cell_type())
    }
}

impl PartialOrd for CellIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct Key(Bytes);

impl Deref for Key {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Timestamp(u64);

impl Deref for Timestamp {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ColumnName(Bytes);

impl Deref for ColumnName {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Value(Bytes);

impl Deref for Value {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum CellType {
    Minimum = 0,
    Put = 4,
    Delete = 8,
    DeleteColumn = 16,
    Maximum = 255,
}

impl TryFrom<u8> for CellType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CellType::Minimum),
            4 => Ok(CellType::Put),
            8 => Ok(CellType::Delete),
            16 => Ok(CellType::DeleteColumn),
            255 => Ok(CellType::Maximum),
            _ => Err("Invalid value trying to convert u8 to CellType enum."),
        }
    }
}
