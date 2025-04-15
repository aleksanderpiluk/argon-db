use libargondb::PartitionKey;

use crate::shared::Reader;

use super::{
    partition_row::{PartitionRow, PartitionRowReader},
    Partition,
};

pub struct PartitionReader;

impl Reader<Partition> for PartitionReader {
    fn try_read<R: std::io::Read>(reader: &mut R) -> anyhow::Result<Partition> {
        let mut buf = vec![0u8; 4];
        reader.read_exact(&mut buf)?;

        let key_len = u16::from_be_bytes(buf[0..2].try_into().unwrap());
        let rows_count = u16::from_be_bytes(buf[2..4].try_into().unwrap());

        let mut buf = vec![0u8; key_len as usize];
        reader.read_exact(&mut buf)?;

        let mut rows: Vec<PartitionRow> = Vec::with_capacity(rows_count as usize);
        for i in 0..rows_count {
            rows[i as usize] = PartitionRowReader::try_read(reader)?;
        }

        Ok(Partition {
            key: PartitionKey::try_from(buf.into_boxed_slice())?,
            rows: rows.into_boxed_slice(),
        })
    }
}
