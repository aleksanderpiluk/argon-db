pub const ARGONFILE_MAGIC: &[u8] = "ARGNFILE".as_bytes();

pub const ARGONFILE_MAGIC_LEN: i64 = 8;
pub const ARGONFILE_TRAILER_LEN: i64 = 12;

pub struct Pointer {
    offset: u64,
    length: u32,
}

pub struct ArgonfileCell {
    column_id: u16,
    is_deleted: bool,
    timestamp: Option<u64>,
    value: Option<Box<[u8]>>,
}

impl ArgonfileCell {
    fn column_id(&self) -> u16 {
        self.column_id
    }

    fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    fn timestamp(&self) -> Option<u64> {
        self.timestamp
    }
}
