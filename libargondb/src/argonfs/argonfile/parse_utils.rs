use crate::argonfs::argonfile::error::{ArgonfileParseError, ArgonfileParseResult};

pub fn ensure_size(actual: usize, expected: usize) -> ArgonfileParseResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(ArgonfileParseError)
    }
}

pub fn ensure_min_size(actual: usize, expected: usize) -> ArgonfileParseResult<()> {
    if actual >= expected {
        Ok(())
    } else {
        Err(ArgonfileParseError)
    }
}
