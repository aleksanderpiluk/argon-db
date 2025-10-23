pub struct KVConfig {
    pub memtable_size: usize,
}

const DEFAULT_MEMTABLE_SIZE: usize = 1 * 1024 * 1024; // 1MB

impl Default for KVConfig {
    fn default() -> Self {
        KVConfig {
            memtable_size: DEFAULT_MEMTABLE_SIZE,
        }
    }
}

pub struct KVConfigValidator;

impl KVConfigValidator {
    pub fn validate(config: &KVConfig) -> Result<(), ()> {
        todo!()
    }
}
