use super::OperationData;

#[derive(Debug)]
pub struct ListKeyspacesData {}

#[derive(Debug)]
pub struct ListKeyspacesResponse {}

impl OperationData<ListKeyspacesResponse> for ListKeyspacesData {
    fn execute(&self) -> ListKeyspacesResponse {
        ListKeyspacesResponse {}
    }
}
