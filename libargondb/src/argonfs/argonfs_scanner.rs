use std::{fs, io, path::PathBuf, sync::Arc};

use crate::{
    ArgonFsConfig,
    argonfs::{io_subsystem::IOSubsystem, path_factory::ArgonFsPathFactory},
    kv::KVSSTableReader,
};

pub struct ArgonFsScanner {
    io_subsystem: Arc<IOSubsystem>,
    path_factory: ArgonFsPathFactory,
}

impl ArgonFsScanner {
    pub fn new(config: &ArgonFsConfig, io_subsystem: Arc<IOSubsystem>) -> Self {
        let path_factory = ArgonFsPathFactory::new(config.clone());

        Self {
            io_subsystem,
            path_factory,
        }
    }

    pub fn scan_table(&self, table_name: &str) -> Result<ArgonFsScanTableResult, io::Error> {
        let table_dir = self.path_factory.table_dir(table_name);

        let result_dir = self.io_subsystem.platform_io_adapter().scan_dir(table_dir);

        let sstables = vec![];

        // let table_dir_exists = match fs::exists(&table_dir) {
        //     Ok(exists) => exists,
        //     Err(err) => {
        //         return Err(err);
        //     }
        // };

        // if table_dir_exists {
        //     match fs::read_dir(&table_dir) {
        //         Ok(dir_iter) => {
        //             for entry in dir_iter {
        //                 let entry = entry?;
        //             }
        //         }
        //         Err(err) => {
        //             return Err(err);
        //         }
        //     };
        // }

        Ok(ArgonFsScanTableResult { sstables })
    }
}

pub struct ArgonFsScanTableResult {
    pub sstables: Vec<Arc<Box<dyn KVSSTableReader + Send + Sync>>>,
}
