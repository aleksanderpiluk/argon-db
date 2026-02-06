mod numeric;

pub use numeric::{I8, I16, I32, I64, U8, U16, U32, U64};

trait Type<T = Self> {
    const TYPE_ID: TypeKind;

    fn decode(data: &[u8]) -> Result<T, TypeError>;
    fn encode<'a>(value: &'a Self) -> Result<std::borrow::Cow<'a, [u8]>, TypeError>;

    fn eq(l: &[u8], r: &[u8]) -> Result<bool, TypeError>;
    fn cmp(l: &[u8], r: &[u8]) -> Result<std::cmp::Ordering, TypeError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    // ArrayF64,
    // ArrayI8,
    // ArrayI16,
    // ArrayI32,
    // ArrayI64,
    // ArrayU8,
    // ArrayU16,
    // ArrayU32,
    // ArrayU64,
    // Bytes,
    // F64,
    I8,
    I16,
    I32,
    I64,
    // Text,
    U8,
    U16,
    U32,
    U64,
}

impl TypeKind {
    pub fn eq(&self, l: &[u8], r: &[u8]) -> Result<bool, TypeError> {
        match self {
            Self::I8 => I8::eq(l, r),
            Self::I16 => I16::eq(l, r),
            Self::I32 => I32::eq(l, r),
            Self::I64 => I64::eq(l, r),
            Self::U8 => U8::eq(l, r),
            Self::U16 => U16::eq(l, r),
            Self::U32 => U32::eq(l, r),
            Self::U64 => U64::eq(l, r),
        }
    }

    pub fn cmp(&self, l: &[u8], r: &[u8]) -> Result<std::cmp::Ordering, TypeError> {
        match self {
            Self::I8 => I8::cmp(l, r),
            Self::I16 => I16::cmp(l, r),
            Self::I32 => I32::cmp(l, r),
            Self::I64 => I64::cmp(l, r),
            Self::U8 => U8::cmp(l, r),
            Self::U16 => U16::cmp(l, r),
            Self::U32 => U32::cmp(l, r),
            Self::U64 => U64::cmp(l, r),
        }
    }
}

#[derive(Debug)]
pub struct TypeError {
    kind: TypeKind,
    msg: String,
}

impl TypeError {
    pub fn new(kind: TypeKind, msg: impl AsRef<str>) -> Self {
        Self {
            kind,
            msg: msg.as_ref().to_string(),
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeError - kind: {:?}, msg: {}", self.kind, self.msg)
    }
}

impl std::error::Error for TypeError {}
