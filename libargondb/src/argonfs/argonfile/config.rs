pub struct ArgonfileConfig {
    pub data_block_size: usize,
}

const DEFAULT_DATA_BLOCK_SIZE: usize = 8 * 1024;

impl Default for ArgonfileConfig {
    fn default() -> Self {
        Self {
            data_block_size: DEFAULT_DATA_BLOCK_SIZE,
        }
    }
}

struct ArgonfileConfigValidator;

impl ArgonfileConfigValidator {
    pub fn validate(config: &ArgonfileConfig) {
        todo!()
    }
}
