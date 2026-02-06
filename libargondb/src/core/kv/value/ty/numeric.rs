macro_rules! type_impl {
    ($strct:ident,$kind:expr,$numty:ty) => {
        pub struct $strct($numty);

        impl super::Type for $strct {
            const TYPE_ID: super::TypeKind = $kind;

            fn decode(bytes: &[u8]) -> Result<Self, super::TypeError> {
                let bytes_array = bytes
                    .try_into()
                    .map_err(|_| super::TypeError::new(Self::TYPE_ID, "invalid buffer size"))?;

                Ok(Self(<$numty>::from_le_bytes(bytes_array)))
            }

            fn encode<'a>(value: &'a Self) -> Result<std::borrow::Cow<'a, [u8]>, super::TypeError> {
                let bytes = <$numty>::to_le_bytes(value.0);

                Ok(std::borrow::Cow::Owned(bytes.to_vec()))
            }

            fn eq(l: &[u8], r: &[u8]) -> Result<bool, super::TypeError> {
                let l = Self::decode(l)?;
                let r = Self::decode(r)?;

                Ok(l.0 == r.0)
            }

            fn cmp(l: &[u8], r: &[u8]) -> Result<std::cmp::Ordering, super::TypeError> {
                let l = Self::decode(l)?;
                let r = Self::decode(r)?;

                Ok(l.0.cmp(&r.0))
            }
        }
    };
}

type_impl!(U8, super::TypeKind::U8, u8);
type_impl!(U16, super::TypeKind::U16, u16);
type_impl!(U32, super::TypeKind::U32, u32);
type_impl!(U64, super::TypeKind::U64, u64);

type_impl!(I8, super::TypeKind::I8, i8);
type_impl!(I16, super::TypeKind::I16, i16);
type_impl!(I32, super::TypeKind::I32, i32);
type_impl!(I64, super::TypeKind::I64, i64);

type_impl!(F64, super::TypeKind::F64, f64);
