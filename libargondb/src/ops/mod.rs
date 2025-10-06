use std::sync::Arc;

enum OpError {
    TableNotFound,
}

trait Mutation {
    fn execute(self, ctx: &OpCtx) -> Result<(), OpError>;
}

struct OpCtx {}

impl OpCtx {}

struct Insert {
    table_name: String,
    columns: Box<[String]>,
    values: Box<[String]>,
}

impl Mutation for Insert {
    fn execute(self, ctx: &OpCtx) -> Result<(), OpError> {
        todo!()
    }
}

struct CreateTable {
    table_name: String,
    columns: (),
    primary_key: (),
}

struct QueryCtx {
    db: Arc<()>, //DbStateSnapshot
}
