use std::sync::{Arc, atomic::AtomicBool};

use signal_hook::iterator::SignalsInfo;
use signal_hook::iterator::exfiltrator::WithOrigin;
use signal_hook::{consts::TERM_SIGNALS, flag};

use crate::errors::{CriticalResult, OrCriticalError};
use crate::supervisor::Lifecycle;

pub fn handle_signals(lifecycle: &Lifecycle) -> CriticalResult<()> {
    let term_now = Arc::new(AtomicBool::new(false));

    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, term_now.clone()).ok_or_critical_err()?;

        flag::register(*sig, term_now.clone()).ok_or_critical_err()?;
    }

    let mut signals = SignalsInfo::<WithOrigin>::new(TERM_SIGNALS).ok_or_critical_err()?;

    for info in &mut signals {
        match info.signal {
            term_signal => {
                eprintln!("received signal {}", term_signal);
                assert!(TERM_SIGNALS.contains(&term_signal));

                break;
            }
        }
    }

    // lifecycle.close();

    Ok(())
}
