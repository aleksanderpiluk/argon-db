use std::io::Write;

use async_trait::async_trait;

use super::{
    block::ArgonfileBlockBuilder, block_identifier::BLOCK_IDENTIFIER_DATA,
    block_ptr::ArgonfileBlockPointer, config::ArgonfileConfig, error::ArgonfileBuilderError,
    magic::ArgonfileMagicWriter, row::ArgonfileRowBuilder, stats::ArgonfileStatsBuilder,
    summary::ArgonfileSummaryBuilder, utils::ArgonfileOffsetCountingWriteWrapper,
};
use crate::{
    argonfs::argonfile::{
        Trailer,
        checksum::{ChecksumAlgoResolver, ChecksumType},
        compression::{CompressionAlgoResolver, CompressionType},
    },
    kv::{KVSSTableBuilder, mutation::KVMutation},
};

struct ArgonfileBuilder<'a, W: Write + Send> {
    config: &'a ArgonfileBuilderConfig,
    orchestrator: BlocksBuildingOrchestrator<'a, W>,
}

impl<'a, W: Write + Send> ArgonfileBuilder<'a, W> {
    pub fn begin(
        config: &'a ArgonfileBuilderConfig,
        writer: W,
        mutations_count: usize,
    ) -> Result<Self, ()> {
        let summary_builder = ArgonfileSummaryBuilder::new();
        let stats_builder = ArgonfileStatsBuilder::new(mutations_count);

        let mut writer = ArgonfileOffsetCountingWriteWrapper::new(writer);
        ArgonfileMagicWriter::write(&mut writer).unwrap();

        let orchestrator =
            BlocksBuildingOrchestrator::new(config, writer, stats_builder, summary_builder);

        Ok(Self {
            config,
            orchestrator,
        })
    }

    pub fn finalize(self) -> Result<W, ArgonfileBuilderError> {
        let orchestrator = self.orchestrator;
        let (mut writer, summary_block_ptr, stats_block_ptr) = orchestrator.end()?;

        Trailer::serialize(
            &mut writer,
            &Trailer {
                sstable_id: todo!(),
                summary_block_ptr,
                stats_block_ptr,
            },
        )?;

        Ok(writer.into_inner())
    }
}

#[async_trait]
impl<'a, W: Write + Send> KVSSTableBuilder for ArgonfileBuilder<'a, W> {
    async fn add_mutation<T: KVMutation + Send + Sync>(&mut self, mutation: &T) -> Result<(), ()> {
        self.orchestrator.add_mutation(mutation).unwrap();

        Ok(())
    }
}

struct BlocksBuildingOrchestrator<'a, W: Write> {
    config: &'a ArgonfileBuilderConfig,
    writer: ArgonfileOffsetCountingWriteWrapper<W>,

    stats_builder: ArgonfileStatsBuilder,
    summary_builder: ArgonfileSummaryBuilder,

    block_builder: Option<ArgonfileBlockBuilder>,
    row_builder: Option<ArgonfileRowBuilder>,
}

impl<'a, W: Write> BlocksBuildingOrchestrator<'a, W> {
    fn new(
        config: &'a ArgonfileBuilderConfig,
        write: ArgonfileOffsetCountingWriteWrapper<W>,
        stats_builder: ArgonfileStatsBuilder,
        summary_builder: ArgonfileSummaryBuilder,
    ) -> Self {
        Self {
            config,
            writer: write,

            stats_builder,
            summary_builder,

            block_builder: None,
            row_builder: None,
        }
    }

    fn add_mutation(&mut self, mutation: &impl KVMutation) -> Result<(), ArgonfileBuilderError> {
        self.prepare_row(mutation)?;

        self.stats_builder.add_mutation(mutation);
        self.row_builder.as_mut().unwrap().add_mutation(mutation);

        Ok(())
    }

    fn end(
        mut self,
    ) -> Result<
        (
            ArgonfileOffsetCountingWriteWrapper<W>,
            ArgonfileBlockPointer,
            ArgonfileBlockPointer,
        ),
        ArgonfileBuilderError,
    > {
        self.end_row()?;
        self.flush_block()?;

        let summary_block_ptr = self.summary_builder.build(&mut self.writer);
        let stats_block_ptr = self.stats_builder.build(&mut self.writer);

        Ok((self.writer, summary_block_ptr, stats_block_ptr))
    }

    fn prepare_row(&mut self, mutation: &impl KVMutation) -> Result<(), ArgonfileBuilderError> {
        self.ensure_block_existence(mutation)?;

        if let Some(row_builder) = &self.row_builder {
            if !row_builder.belongs_to_row(mutation) {
                self.end_row_and_start_new(mutation)?;
            }
        } else {
            self.start_new_row(mutation);
        }

        Ok(())
    }

    fn end_row_and_start_new(
        &mut self,
        mutation: &impl KVMutation,
    ) -> Result<(), ArgonfileBuilderError> {
        self.end_row()?;
        self.flush_block_if_necessary()?;
        self.start_new_row(mutation);
        Ok(())
    }

    fn end_row(&mut self) -> Result<(), ArgonfileBuilderError> {
        let row_builder = self
            .row_builder
            .take()
            .ok_or(ArgonfileBuilderError::AssertionError)?;
        let block_builder = self
            .block_builder
            .as_mut()
            .ok_or(ArgonfileBuilderError::AssertionError)?;

        row_builder.end_row(block_builder)?;
        Ok(())
    }

    fn start_new_row(&mut self, mutation: &impl KVMutation) {
        let primary_key = Box::from(mutation.primary_key());
        self.row_builder = Some(ArgonfileRowBuilder::new(primary_key));
    }

    fn ensure_block_existence(
        &mut self,
        mutation: &impl KVMutation,
    ) -> Result<(), ArgonfileBuilderError> {
        if let None = self.block_builder {
            self.new_data_block(mutation);
        }

        Ok(())
    }

    fn flush_block_if_necessary(&mut self) -> Result<(), ArgonfileBuilderError> {
        let block_builder = self
            .block_builder
            .as_ref()
            .ok_or(ArgonfileBuilderError::AssertionError)?;

        if block_builder.is_desired_size_exceeded() {
            self.flush_block();
        }

        Ok(())
    }

    fn flush_block(&mut self) -> Result<(), ArgonfileBuilderError> {
        let block_builder = self
            .block_builder
            .take()
            .ok_or(ArgonfileBuilderError::AssertionError)?;

        let checksum_algo = ChecksumAlgoResolver::for_checksum_type(ChecksumType::CRC32);
        let compression_algo =
            CompressionAlgoResolver::for_compression_type(CompressionType::Uncompressed);

        let block_ptr = block_builder.build(
            &mut self.writer,
            BLOCK_IDENTIFIER_DATA,
            &checksum_algo,
            &compression_algo,
        )?;

        self.summary_builder.finish_block_with_ptr(block_ptr);

        Ok(())
    }

    fn new_data_block(&mut self, mutation: &impl KVMutation) {
        self.block_builder = Some(ArgonfileBlockBuilder::new(self.config.data_block_size));
        self.summary_builder.next_block_with_min_key(mutation);
    }
}

pub struct ArgonfileBuilderConfig {
    pub data_block_size: usize,
}

const DEFAULT_DATA_BLOCK_SIZE: usize = 8 * 1024;

impl Default for ArgonfileBuilderConfig {
    fn default() -> Self {
        Self {
            data_block_size: DEFAULT_DATA_BLOCK_SIZE,
        }
    }
}
