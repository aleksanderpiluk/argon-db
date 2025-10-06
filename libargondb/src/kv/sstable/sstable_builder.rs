use crate::kv::mutation::Mutation;

pub struct SSTableBuilder;

impl SSTableBuilder {
    pub fn write_mutation(&mut self, mutation: &dyn Mutation) {
        todo!()
    }
}
