use crate::kv::column_type::ColumnType;

pub struct Bytes;

impl ColumnType for Bytes {
    fn eq(this: &[u8], that: &[u8]) -> bool {
        this.eq(that)
    }

    fn cmp(this: &[u8], that: &[u8]) -> std::cmp::Ordering {
        this.cmp(that)
    }
}
