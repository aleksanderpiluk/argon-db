use crate::base::module::{Module, ModuleName};

pub struct BasicConnector {}

impl BasicConnector {
    pub fn new() -> Self {
        Self {}
    }
}

impl Module for BasicConnector {
    fn module_name<'a>(&self) -> ModuleName<'a> {
        ModuleName::from("connector_basic")
    }

    fn setup(&self) -> Result<(), ()> {
        Ok(())
    }
}
