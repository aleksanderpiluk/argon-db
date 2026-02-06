use super::super::value;

pub struct Schema(Vec<value::TypeKind>);

impl Schema {
    const MAX_COLUMNS_IN_PRIMARY_KEY: u8 = 128;

    pub fn new(columns: Vec<value::TypeKind>) -> Result<Self, ()> {
        if columns.len() > Self::MAX_COLUMNS_IN_PRIMARY_KEY as usize {
            return Err(());
        }

        Ok(Self(columns))
    }

    pub fn column_count(&self) -> u8 {
        debug_assert!(self.0.len() <= Self::MAX_COLUMNS_IN_PRIMARY_KEY as usize);

        self.0.len() as u8
    }

    pub fn get_column(&self, column_index: usize) -> Result<value::TypeKind, ()> {
        debug_assert!(self.0.len() <= Self::MAX_COLUMNS_IN_PRIMARY_KEY as usize);

        if column_index <= self.0.len() {
            return Err(());
        }

        Ok(self.0[column_index])
    }
}
