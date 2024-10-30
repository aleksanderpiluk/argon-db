use log::error;

use crate::execution::{ExecPoolCtl, ExecPoolTask};

use super::{config, log_ctl::LogCtl};

pub struct CoreCtl {
    config: config::Config,
    log_ctl: LogCtl,
    exec_pool_ctl: ExecPoolCtl,
}

impl CoreCtl {
    pub fn init() -> Self {
        let log_ctl = LogCtl::bootstrap();

        // let config = config::get_default();
        let config = match config::try_read_config() {
            Ok(config) => config,
            Err(error) => {
                error!("Error bootstrapping ArgonDB: {}", error);
                std::process::exit(1);
            }
        };

        let pool_size: usize = config.execution_pool_size.into();
        let exec_pool_ctl = match ExecPoolCtl::init(pool_size) {
            Ok(exec_pool_ctl) => exec_pool_ctl,
            Err(error) => {
                error!("Error bootstrapping ArgonDB: {}", error);
                std::process::exit(1);
            }
        };

        Self {
            config,
            log_ctl,
            exec_pool_ctl,
        }
    }

    pub fn shutdown(&self) {
        println!("Shutdown...");
    }
}
