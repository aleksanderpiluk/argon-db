mod ty;

pub use ty::TypeKind;

pub struct Value<'a> {
    data: std::borrow::Cow<'a, [u8]>,
    kind: ty::TypeKind,
}

impl<'a> Value<'a> {
    pub fn new(data: &'a [u8], kind: ty::TypeKind) -> Result<Self, ()> {
        Ok(Self {
            data: std::borrow::Cow::Borrowed(data),
            kind,
        })
    }

    pub fn eq(&self, other: &Self) -> Result<bool, ValueComparisonError> {
        if self.kind != other.kind {
            return Err(ValueComparisonError::TypeMismatch(self.kind, other.kind));
        }

        self.kind
            .eq(&self.data, &other.data)
            .map_err(|err| ValueComparisonError::TypeError(err))
    }

    pub fn cmp(&self, other: &Self) -> Result<std::cmp::Ordering, ValueComparisonError> {
        if self.kind != other.kind {
            return Err(ValueComparisonError::TypeMismatch(self.kind, other.kind));
        }

        self.kind
            .cmp(&self.data, &other.data)
            .map_err(|err| ValueComparisonError::TypeError(err))
    }
}

pub enum ValueComparisonError {
    TypeError(ty::TypeError),
    TypeMismatch(ty::TypeKind, ty::TypeKind),
}
