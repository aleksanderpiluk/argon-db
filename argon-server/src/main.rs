extern crate num_cpus;

mod core;
mod execution;
mod memstore;
mod table;

use self::core::CoreCtl;

fn main() {
    CoreCtl::init();
}
