mod object_id_generator;

pub use object_id_generator::ObjectIdGenerator;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ObjectId(pub u64);

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
