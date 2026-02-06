use super::super::base;
use super::super::primary_key;

pub struct MutationComparator<S: AsRef<primary_key::Schema>> {
    schema: S,
}

impl<S: AsRef<primary_key::Schema>> MutationComparator<S> {
    pub fn new(schema: S) -> Self {
        Self { schema }
    }
}

impl<T, R, S: AsRef<primary_key::Schema>> base::Comparator<(), T, R> for MutationComparator<S> {
    fn cmp(&self, l: &T, r: &R) -> Result<std::cmp::Ordering, ()> {
        todo!()
    }

    fn eq(&self, l: &T, r: &R) -> Result<bool, ()> {
        todo!()
    }
}
