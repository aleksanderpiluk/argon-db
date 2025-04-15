use anyhow::Context;

use crate::{pointer::Pointer, shared::Reader};

pub struct Trailer {
    compression_coded: u16,
    min_key_size: u16,
    max_key_size: u16,
    summary_block: Pointer,
    filter_block: Pointer,
}

pub struct TrailerReader;

impl Reader<Trailer> for TrailerReader {
    fn try_read<R: std::io::Read>(reader: &mut R) -> anyhow::Result<Trailer> {
        let mut buf = vec![0u8; 30];
        reader
            .read_exact(&mut buf)
            .with_context(|| format!("Failed to read trailer"))?;

        todo!();
        // Ok(Trailer {
        //     compression_coded: u16::from_be_bytes(buf[0..2].try_into()?),
        //     summary_block: (),
        //     filter_block: (),
        // })
    }
}

pub struct TrailerWriter;

impl TrailerWriter {}
