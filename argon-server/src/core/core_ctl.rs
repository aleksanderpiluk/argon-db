use crate::execution::ExecPoolCtl;

use super::{config, log};

pub struct CoreCtl {}

impl CoreCtl {
    pub fn init() {
        let config = config::get_default();
        // let config = match config::try_read_config() {
        //     Ok(config) => config,
        //     Err(error) => {
        //         log::log_bootstrap_err(error);
        //         std::process::exit(1);
        //     }
        // };

        let pool_size: usize = config.execution_pool_size.into();
        let exec_pool_ctl = match ExecPoolCtl::init(pool_size) {
            Ok(exec_pool_ctl) => exec_pool_ctl,
            Err(error) => {
                log::log_bootstrap_err(error);
                std::process::exit(1);
            }
        };
    }
}
