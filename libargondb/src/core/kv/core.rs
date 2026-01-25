pub trait Comparator<E, L, R = L> {
    fn cmp(&self, l: &L, r: &R) -> Result<std::cmp::Ordering, E>;
    fn eq(&self, l: &L, r: &R) -> Result<bool, E>;
}
