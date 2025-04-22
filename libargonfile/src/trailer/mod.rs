mod trailer_writer;

pub use trailer_writer::TrailerWriter;

use crate::{pointer::Pointer, shared::Reader, stats::Stats};

pub struct Trailer {
    compression_coded: u16,
    stats: Stats,
    summary_block: Pointer,
    filter_block: Pointer,
}

impl Trailer {
    pub fn new(
        compression_coded: u16,
        stats: Stats,
        summary_block: Pointer,
        filter_block: Pointer,
    ) -> Self {
        Self {
            compression_coded,
            stats,
            summary_block,
            filter_block,
        }
    }
}

// pub struct TrailerReader;

// impl Reader<Trailer> for TrailerReader {
//     fn try_read<R: std::io::Read>(reader: &mut R) -> anyhow::Result<Trailer> {
//         let mut buf = vec![0u8; 30];
//         reader
//             .read_exact(&mut buf)
//             .with_context(|| format!("Failed to read trailer"))?;

//         todo!();
//         // Ok(Trailer {
//         //     compression_coded: u16::from_be_bytes(buf[0..2].try_into()?),
//         //     summary_block: (),
//         //     filter_block: (),
//         // })
//     }
// }

// pub struct TrailerWriter;

// impl TrailerWriter {}
