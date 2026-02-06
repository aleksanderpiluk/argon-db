use crate::kv::primary_key::PrimaryKey;

pub struct RowScanParams {
    pub key: PrimaryKey<'static>,
}
