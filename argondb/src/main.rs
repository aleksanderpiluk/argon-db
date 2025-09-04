use libargonrt::{
    argonrt_setup,
    modules::{basic_connector::BasicConnector, storage::DefaultStorageModule},
};

fn main() {
    let mut rt = argonrt_setup();

    rt.load_module(DefaultStorageModule::new());
    rt.load_module(BasicConnector::new());
}
