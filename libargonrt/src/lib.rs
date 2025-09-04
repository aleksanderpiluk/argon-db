mod base;
mod cee;
mod limits;
pub mod modules;
pub mod runtime;

use crate::runtime::{Runtime, systems::log::LogSystem};

pub fn argonrt_setup() -> Runtime {
    let log = LogSystem::init();

    Runtime::new()
}

#[inline]
pub fn rt() {}
