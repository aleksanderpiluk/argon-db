pub struct ConfigProvider {
    segment_size: usize,
}

impl Default for ConfigProvider {
    fn default() -> Self {
        Self {
            segment_size: SEGMENT_SIZE_DEFAULT,
        }
    }
}

impl ConfigProvider {
    pub fn set_segment_size(&mut self, segment_size: usize) -> Result<(), ConfigProviderError> {
        if segment_size > SEGMENT_SIZE_MAX {
            return Err(ConfigProviderError::InvalidValue(format!(
                "Value exceeds max allowed value {}",
                SEGMENT_SIZE_MAX
            )));
        }

        self.segment_size = segment_size;
        Ok(())
    }
}

#[derive(Debug)]
enum ConfigProviderError {
    InvalidValue(String),
}

/// The default size of single WAL segment.
const SEGMENT_SIZE_DEFAULT: usize = 16 * 1024 * 1024;
/// The maximum size of single WAL segment
const SEGMENT_SIZE_MAX: usize = 512 * 1024 * 1024;
