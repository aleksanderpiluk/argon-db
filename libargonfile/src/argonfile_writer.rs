use std::io::Write;

use anyhow::{Ok, Result};
use libargondb::TableMutation;

use crate::{
    block::{Block, BlockBuilder, BlockIdentifier, BlockWriter},
    bloom::BloomBuilder,
    index::{IndexBuilder, IndexEntry, IndexEntryWriter, SummaryBuilder, SummaryEntry},
    partition::{Partition, PartitionBuilder, PartitionWriter},
    shared::{self, PositionedWriter, Writer},
    stats::StatsBuilder,
    trailer::{Trailer, TrailerWriter},
};

/// An `ArgonfileWriter` serializes provided table mutations to argonfile file.
/// During write, it automatically builds data index and other blocks.
#[derive(Debug)]
pub struct ArgonfileWriter<W: Write> {
    inner: ArgonfileDataWriter<W>,
}

impl<W: Write> ArgonfileWriter<W> {
    pub fn start(writer: W, est_partition_count: usize, mutation: &TableMutation) -> Result<Self> {
        let mut writer = PositionedWriter::new(writer);

        /* Magic bytes at the begin of the file */
        writer.write(shared::ARGONFILE_MAGIC)?;

        Ok(Self {
            inner: ArgonfileDataWriter::new(writer, est_partition_count, mutation),
        })
    }

    pub fn write_mutation(self, mutation: &TableMutation) -> Result<()> {
        self.inner.write_table_mutation(mutation)
    }

    pub fn close(self) -> Result<W> {
        let mut writer = self.inner.close()?;
        /* Magic bytes at the end of the file */
        writer.write(shared::ARGONFILE_MAGIC)?;

        Ok(writer.into())
    }
}

#[derive(Debug)]
struct ArgonfileDataWriter<W: Write> {
    writer: PositionedWriter<W>,
    partition_builder: PartitionBuilder,
    index_builder: IndexBuilder,
    block_builder: BlockBuilder,
    bloom_builder: BloomBuilder,
    stats_builder: StatsBuilder,
}

impl<W: Write> ArgonfileDataWriter<W> {
    fn new(
        writer: PositionedWriter<W>,
        est_partition_count: usize,
        mutation: &TableMutation,
    ) -> Self {
        Self {
            writer,
            block_builder: BlockBuilder::new(
                BlockIdentifier::DATA_BLOCK,
                Block::BLOCK_CONTENT_SIZE,
            ),
            index_builder: IndexBuilder::new(),
            partition_builder: PartitionBuilder::new(mutation),
            bloom_builder: BloomBuilder::new(est_partition_count).unwrap(),
            stats_builder: StatsBuilder::new(&mutation.key().partition_key()),
        }
    }

    pub fn write_table_mutation(mut self, mutation: &TableMutation) -> Result<()> {
        if let Some(partition) = self.partition_builder.next(mutation) {
            Self::_write_partition(
                &mut self.block_builder,
                &mut self.bloom_builder,
                &mut self.index_builder,
                partition,
            )?;

            if let Some(data_block) = self.block_builder.next() {
                let block_ptr = BlockWriter::try_write(&mut self.writer, &data_block)?;
                self.index_builder.commit_block(block_ptr);
            }
        }

        Ok(())
    }

    pub fn close(mut self) -> Result<W> {
        let partition = PartitionBuilder::close(self.partition_builder);

        let stats = self.stats_builder.close(partition.key());

        Self::_write_partition(
            &mut self.block_builder,
            &mut self.bloom_builder,
            &mut self.index_builder,
            partition,
        )?;

        let block = BlockBuilder::close(self.block_builder);
        let block_ptr = if let Some(block) = block {
            Some(BlockWriter::try_write(&mut self.writer, &block)?)
        } else {
            None
        };

        let indices = self.index_builder.close(block_ptr);
        let mut index_block_builder =
            BlockBuilder::new(BlockIdentifier::INDEX_BLOCK, Block::BLOCK_CONTENT_SIZE);

        let mut summary_entries: Vec<SummaryEntry> = vec![];
        let mut block_first_index: Option<IndexEntry> = None;
        for index in indices {
            if block_first_index.is_none() {
                block_first_index = Some(index.clone());
            }

            IndexEntryWriter::try_write(index_block_builder.as_mut(), &index)?;

            if let Some(block) = index_block_builder.next() {
                let block_ptr = BlockWriter::try_write(&mut self.writer, &block)?;

                summary_entries.push(SummaryEntry::new(
                    block_ptr,
                    block_first_index.unwrap().into_key(),
                ));
                block_first_index = None;
            }
        }
        if let Some(block) = BlockBuilder::close(index_block_builder) {
            let block_ptr = BlockWriter::try_write(&mut self.writer, &block)?;

            summary_entries.push(SummaryEntry::new(
                block_ptr,
                block_first_index.unwrap().into_key(),
            ));
        }

        let summary_block = SummaryBuilder::block_from_entries(summary_entries)?;
        let summary_block_ptr = BlockWriter::try_write(&mut self.writer, &summary_block)?;

        let bloom_block = self.bloom_builder.close()?;
        let bloom_block_ptr = BlockWriter::try_write(&mut self.writer, &bloom_block)?;

        let trailer = Trailer::new(0u16, stats, summary_block_ptr, bloom_block_ptr);
        TrailerWriter::try_write(&mut self.writer, &trailer)?;

        Ok(self.writer.into())
    }

    fn _write_partition(
        block_builder: &mut BlockBuilder,
        bloom_builder: &mut BloomBuilder,
        index_builder: &mut IndexBuilder,
        partition: Partition,
    ) -> Result<()> {
        // Pointer to partition with in-block offset
        let pointer = PartitionWriter::try_write(block_builder.as_mut(), &partition)?;

        bloom_builder.add_partition_key(partition.key());

        index_builder.add_entry_in_block(partition.into_key().into(), pointer.offset() as u32);

        Ok(())
    }
}
