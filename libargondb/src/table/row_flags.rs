struct RowFlags<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for RowFlags<'a> {
    fn from(value: &'a [u8]) -> Self {
        assert_eq!(value.len(), 2);

        Self(value)
    }
}
