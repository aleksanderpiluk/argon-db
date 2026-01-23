use std::process;

use crate::errors::CriticalError;

pub fn abort_on_critical_error(err: CriticalError) -> ! {
    eprintln!("{}", err);
    eprintln!("exiting process due to critical error");

    process::exit(1)
}
