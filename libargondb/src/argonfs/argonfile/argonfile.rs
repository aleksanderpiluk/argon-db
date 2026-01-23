use crate::{
    argonfs::{
        argonfile::{
            ArgonfileReader, ArgonfileReaderError, BlockPointer, block::Block, stats::StatsParser,
            summary::SummaryParser,
        },
        fs::BoxFileRef,
    },
    kv::ObjectId,
};

use super::{stats::Stats, summary::SummaryIndex};

pub struct Argonfile {
    pub file_ref: BoxFileRef,
    pub sstable_id: ObjectId,
    pub level: u64,
    pub summary_index: SummaryIndex,
    pub stats: Stats,
}

impl Argonfile {
    pub async fn from_file_ref(file_ref: BoxFileRef) -> Result<Self, ArgonfileReaderError> {
        let file_handle = file_ref.open_read_only().await?;
        let mut reader = ArgonfileReader::new(file_handle);

        let trailer = reader.read_trailer().await?;

        let sstable_id = trailer.sstable_id;
        let level = trailer.level;

        let summary_block = reader.read_block(&trailer.summary_block_ptr).await?;
        let summary_index = SummaryParser::parse(&summary_block.data)?;

        let stats_block = reader.read_block(&trailer.stats_block_ptr).await?;
        let stats = StatsParser::parse(&stats_block.data)?;

        Ok(Self {
            file_ref,
            sstable_id,
            level,
            stats,
            summary_index,
        })
    }

    pub async fn read_block(
        &self,
        block_ptr: &BlockPointer,
    ) -> Result<Block, ArgonfileReaderError> {
        let file_handle = self.file_ref.open_read_only().await?;
        let mut reader = ArgonfileReader::new(file_handle);

        reader.read_block(block_ptr).await
    }
}
