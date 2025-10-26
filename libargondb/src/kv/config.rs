pub struct KVConfig {
    pub memtable_size: usize, // TODO:
    pub mutation_max_size: usize,
}

const DEFAULT_MEMTABLE_SIZE: usize = 1 * 1024 * 1024; // 1MB
const DEFAULT_MUTATION_MAX_SIZE: usize = 16 * 1024; // 16kB

impl Default for KVConfig {
    fn default() -> Self {
        KVConfig {
            memtable_size: DEFAULT_MEMTABLE_SIZE,
            mutation_max_size: DEFAULT_MUTATION_MAX_SIZE,
        }
    }
}

pub struct KVConfigValidator;

impl KVConfigValidator {
    pub fn validate(config: &KVConfig) -> Result<(), ()> {
        todo!()
    }
}
