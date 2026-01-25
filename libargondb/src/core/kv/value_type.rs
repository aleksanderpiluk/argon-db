#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueTypeId {
    ArrayF64,
    ArrayI8,
    ArrayI16,
    ArrayI32,
    ArrayI64,
    ArrayU8,
    ArrayU16,
    ArrayU32,
    ArrayU64,
    Bytes,
    F64,
    I8,
    I16,
    I32,
    I64,
    Text,
    U8,
    U16,
    U32,
    U64,
}

trait ValueType<T = Self> {
    const TYPE_ID: ValueTypeId;

    fn deserialize(data: &[u8]) -> Result<T, ()>;
    fn eq(l: &[u8], r: &[u8]) -> Result<bool, ()>;
}

// struct F64;

// impl ValueType for F64 {
//     fn deserialize(data: &[u8]) -> Result<Self, ()> {}
// }
