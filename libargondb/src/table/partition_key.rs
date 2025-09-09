#[derive(Eq, Ord)]
struct PartitionKey<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for PartitionKey<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl PartialEq for PartitionKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl PartialOrd for PartitionKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}
