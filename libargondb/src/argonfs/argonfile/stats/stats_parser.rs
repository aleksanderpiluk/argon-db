use crate::argonfs::argonfile::{error::ArgonfileParseResult, stats::Stats};

pub struct StatsParser;

impl StatsParser {
    pub fn parse(buf: &[u8]) -> ArgonfileParseResult<Stats> {
        todo!()
    }
}
