use std::hash::Hash;

#[derive(Debug, Clone, Copy)]
pub struct BlockTag;

impl PartialEq for BlockTag {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl Eq for BlockTag {}

impl Hash for BlockTag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        todo!()
    }
}
