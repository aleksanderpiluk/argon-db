struct Value<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for Value<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}
