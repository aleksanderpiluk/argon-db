mod read;

use crate::base::{
    module::{Module, ModuleName},
    storage::StorageModule,
};

pub struct DefaultStorageModule {}

impl DefaultStorageModule {
    pub fn new() -> Self {
        Self {}
    }
}

impl Module for DefaultStorageModule {
    fn module_name<'a>(&self) -> ModuleName<'a> {
        ModuleName::STORAGE
    }

    fn setup(&self) -> Result<(), ()> {
        Ok(())
    }
}

impl StorageModule for DefaultStorageModule {
    fn read_block(&self) -> crate::base::storage::GetBlockResult {
        todo!()
    }
}
