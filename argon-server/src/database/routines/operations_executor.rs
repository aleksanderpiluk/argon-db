use crate::{
    operations::{Operation, OperationData},
    Database,
};

pub struct OperationsExecutor {}

impl OperationsExecutor {
    pub fn routine() {
        loop {
            let op_id = Database::channels().operations().pop();
            let op_id = match op_id {
                None => {
                    continue;
                }
                Some(op) => op,
            };

            let op = Database::operations_store().get(op_id).unwrap();

            match op {
                Operation::ListKeyspaces(data, response) => {
                    let result = data.execute();
                    response(result);
                }
            }
        }
    }
}
