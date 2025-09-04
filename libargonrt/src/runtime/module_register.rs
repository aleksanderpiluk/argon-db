use std::collections::HashMap;

use crate::base::module::{Module, ModuleName};

pub struct ModuleRegister<'a> {
    map: HashMap<ModuleName<'a>, ModuleContainer>,
}

impl<'a> ModuleRegister<'a> {
    pub fn new() -> Self {
        Self {
            map: HashMap::with_capacity(256),
        }
    }

    pub fn try_register(
        &mut self,
        module: Box<dyn Module>,
    ) -> Result<&ModuleContainer, ModuleRegisterError> {
        let module_name = module.module_name();
        if self.map.contains_key(&module_name) {
            return Err(ModuleRegisterError::ModuleAlreadyExist);
        }

        self.map
            .insert(module_name.clone(), ModuleContainer { module });
        Ok(self.map.get(&module_name).unwrap())
    }
}

#[derive(Debug)]
pub enum ModuleRegisterError {
    ModuleAlreadyExist,
}

pub struct ModuleContainer {
    module: Box<dyn Module>,
}

impl ModuleContainer {
    pub fn module(&self) -> &Box<dyn Module> {
        &self.module
    }
}
