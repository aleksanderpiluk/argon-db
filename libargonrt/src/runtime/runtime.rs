use log::info;

use crate::{base::module::Module, runtime::module_register::ModuleRegister};

pub struct Runtime {
    module_register: ModuleRegister<'static>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            module_register: ModuleRegister::new(),
        }
    }

    pub fn load_module<T: Module + 'static>(&mut self, module: T) {
        let module = Box::new(module);
        let container = match self.module_register.try_register(module) {
            Ok(m) => m,
            Err(err) => panic!("module failed to load - reason: {:?}", err),
        };

        match container.module().setup() {
            Ok(_) => {}
            Err(_) => panic!("module failed to load - setup failed"),
        };

        info!("module added: {}", container.module().module_name());
    }
}
