use std::{io::Write, sync::Arc};

use async_trait::async_trait;

use super::{
    error::ArgonfileBuilderError, row::RowBuilder, utils::ArgonfileOffsetCountingWriteWrapper,
};
use crate::{
    argonfs::argonfile::{
        BlockPointer, Trailer,
        block::{
            BLOCK_IDENTIFIER_DATA, BlockBuilder,
            checksum::{ChecksumAlgoResolver, ChecksumType},
            compression::{CompressionAlgoResolver, CompressionType},
        },
        stats::StatsBuilder,
        summary::SummaryBuilder,
    },
    kv::{
        KVRuntimeError, KVRuntimeErrorKind, KVSSTableBuilder, KVScannable, memtable::Memtable,
        mutation::KVMutation,
    },
};

pub struct ArgonfileBuilder;

impl ArgonfileBuilder {
    pub async fn flush_memtable<'a, W: Write + Send + Sync>(
        writer: W,
        memtable: Arc<Memtable>,
    ) -> Result<(), ArgonfileBuilderError> {
        let config = ArgonfileBuilderConfig::default();

        let stats_builder = StatsBuilder::new(memtable.clone())?;
        let summary_builder = SummaryBuilder::new();

        let writer = ArgonfileOffsetCountingWriteWrapper::new(writer);

        let mut orchestrator =
            BlocksBuildingOrchestrator::new(&config, writer, stats_builder, summary_builder);

        memtable
            .flush(&mut orchestrator)
            .await
            .map_err(|e| ArgonfileBuilderError::from_source(e))?;

        let (mut writer, summary_block_ptr, stats_block_ptr) = orchestrator.end()?;

        Trailer::serialize(
            &mut writer,
            &Trailer {
                sstable_id: memtable.object_id,
                summary_block_ptr,
                stats_block_ptr,
            },
        )?;

        writer.into_inner().flush().unwrap();

        Ok(())
    }
}

struct BlocksBuildingOrchestrator<'a, W: Write> {
    config: &'a ArgonfileBuilderConfig,
    writer: ArgonfileOffsetCountingWriteWrapper<W>,

    stats_builder: StatsBuilder,
    summary_builder: SummaryBuilder,

    block_builder: Option<BlockBuilder>,
    row_builder: Option<RowBuilder>,
}

impl<'a, W: Write> BlocksBuildingOrchestrator<'a, W> {
    fn new(
        config: &'a ArgonfileBuilderConfig,
        write: ArgonfileOffsetCountingWriteWrapper<W>,
        stats_builder: StatsBuilder,
        summary_builder: SummaryBuilder,
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

    fn end(
        mut self,
    ) -> Result<
        (
            ArgonfileOffsetCountingWriteWrapper<W>,
            BlockPointer,
            BlockPointer,
        ),
        ArgonfileBuilderError,
    > {
        self.end_row()?;
        self.flush_block()?;

        let summary_block_ptr = self.summary_builder.build(&mut self.writer)?;
        let stats_block_ptr = self.stats_builder.build(&mut self.writer)?;

        Ok((self.writer, summary_block_ptr, stats_block_ptr))
    }

    fn prepare_row(&mut self, mutation: &dyn KVMutation) -> Result<(), ArgonfileBuilderError> {
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

    fn end_row_and_start_new<T: KVMutation + ?Sized>(
        &mut self,
        mutation: &T,
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
            .ok_or(ArgonfileBuilderError::from_msg("no row builder"))?;

        let block_builder = self
            .block_builder
            .as_mut()
            .ok_or(ArgonfileBuilderError::from_msg("no block builder"))?;

        row_builder.end_row(block_builder)?;
        Ok(())
    }

    fn start_new_row<T: KVMutation + ?Sized>(&mut self, mutation: &T) {
        let primary_key = Box::from(mutation.primary_key());
        self.row_builder = Some(RowBuilder::new(primary_key));
    }

    fn ensure_block_existence(
        &mut self,
        mutation: &dyn KVMutation,
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
            .ok_or(ArgonfileBuilderError::from_msg("no block builder"))?;

        if block_builder.is_desired_size_exceeded() {
            self.flush_block();
        }

        Ok(())
    }

    fn flush_block(&mut self) -> Result<(), ArgonfileBuilderError> {
        let block_builder = self
            .block_builder
            .take()
            .ok_or(ArgonfileBuilderError::from_msg("no block builder"))?;

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

    fn new_data_block(&mut self, mutation: &dyn KVMutation) {
        self.block_builder = Some(BlockBuilder::new(self.config.data_block_size));
        self.summary_builder.next_block_with_min_key(mutation);
    }
}

#[async_trait]
impl<'a, W: Write + Send> KVSSTableBuilder for BlocksBuildingOrchestrator<'a, W> {
    async fn add_mutation(
        &mut self,
        mutation: &(dyn KVMutation + Send + Sync),
    ) -> Result<(), KVRuntimeError> {
        self.prepare_row(mutation).unwrap();

        self.stats_builder.add_mutation(mutation);
        self.row_builder
            .as_mut()
            .unwrap()
            .add_mutation(mutation)
            .map_err(|e| KVRuntimeError::with_source(KVRuntimeErrorKind::OperationNotAllowed, e))?;

        Ok(())
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
