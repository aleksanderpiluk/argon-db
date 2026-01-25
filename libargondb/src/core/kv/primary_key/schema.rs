use crate::kv::value_type::ValueTypeId;

pub struct PrimaryKeySchema(Vec<ValueTypeId>);

impl PrimaryKeySchema {
    const MAX_COLUMNS_IN_PRIMARY_KEY: u8 = 128;

    pub fn new(columns: Vec<ValueTypeId>) -> Result<Self, ()> {
        if columns.len() > Self::MAX_COLUMNS_IN_PRIMARY_KEY as usize {
            return Err(());
        }

        Ok(Self(columns))
    }

    pub fn column_count(&self) -> u8 {
        debug_assert!(self.0.len() <= Self::MAX_COLUMNS_IN_PRIMARY_KEY as usize);

        self.0.len() as u8
    }

    pub fn get_column(&self, column_index: usize) -> Result<ValueTypeId, ()> {
        debug_assert!(self.0.len() <= Self::MAX_COLUMNS_IN_PRIMARY_KEY as usize);

        if column_index <= self.0.len() {
            return Err(());
        }

        Ok(self.0[column_index])
    }
}
