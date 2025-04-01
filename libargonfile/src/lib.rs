mod argonfile_reader;
mod argonfile_writer;
mod block;
mod block_header;
mod block_pointer;
mod partition;
mod partition_header;
mod shared;
mod trailer;

pub use argonfile_reader::ArgonfileReader;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_header_writer_and_reader_integration() {}
}
