use std::io::Write;

use anyhow::Ok;

use crate::{
    pointer::{Pointer, PointerWriter},
    shared::Writer,
};

use super::Trailer;

pub struct TrailerWriter;

impl Writer<Trailer> for TrailerWriter {
    fn try_write<W: std::io::Write>(
        writer: &mut crate::shared::PositionedWriter<W>,
        trailer: &Trailer,
    ) -> anyhow::Result<crate::pointer::Pointer> {
        let offset = writer.get_position();

        PointerWriter::try_write(writer, &trailer.summary_block)?;
        PointerWriter::try_write(writer, &trailer.filter_block)?;

        writer.write(&u16::to_be_bytes(trailer.compression_coded))?;
        writer.write(&u16::to_be_bytes(trailer.stats.min_key().len() as u16))?;
        writer.write(&u16::to_be_bytes(trailer.stats.max_key().len() as u16))?;

        writer.write(trailer.stats.min_key().as_ref())?;
        writer.write(trailer.stats.max_key().as_ref())?;

        let size = writer.get_position() - offset;
        Ok(Pointer::new(offset, size))
    }
}
