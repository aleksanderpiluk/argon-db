use std::{fs, io, path::PathBuf};

use crate::argonfs::path_factory::ArgonFsPathFactory;

pub struct ArgonFsFsScan {}

impl ArgonFsFsScan {
    pub fn scan_table(
        path_factory: &ArgonFsPathFactory,
        table_name: &str,
    ) -> Result<ArgonFsBootScanTableResult, io::Error> {
        let table_dir = path_factory.table_dir(table_name);

        let sstables = vec![];

        let table_dir_exists = match fs::exists(&table_dir) {
            Ok(exists) => exists,
            Err(err) => {
                return Err(err);
            }
        };

        if table_dir_exists {
            match fs::read_dir(&table_dir) {
                Ok(dir_iter) => {
                    for entry in dir_iter {
                        let entry = entry?;
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            };
        }

        Ok(ArgonFsBootScanTableResult { sstables })
    }
}

struct ArgonFsBootScanTableResult {
    pub sstables: Vec<PathBuf>,
}
