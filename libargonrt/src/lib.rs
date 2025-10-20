pub mod base;
mod cee;
mod clock;
mod limits;
pub mod modules;
mod platform;
pub mod runtime;
pub mod utils;

use crate::runtime::{Runtime, systems::log::LogSystem};

pub use clock::Clock;

pub fn argonrt_setup() -> Runtime {
    let log = LogSystem::init();

    Runtime::new()
}

#[inline]
pub fn rt() {}
