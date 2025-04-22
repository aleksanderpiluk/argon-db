mod list_keyspaces;
mod operations_store;

pub use list_keyspaces::{ListKeyspacesData, ListKeyspacesResponse};
pub use operations_store::OperationsStore;

pub enum Operation {
    ListKeyspaces(ListKeyspacesData, OperationResponse<ListKeyspacesResponse>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationId(u64);

pub trait OperationData<T> {
    fn execute(&self) -> T;
}

type OperationResponse<T> = Box<dyn FnOnce(T) + Send + Sync>;
