mod algo;
mod compression_algo;
mod compression_algo_resolver;
mod compression_error;
mod compression_type;

pub use compression_algo::CompressionAlgo;
pub use compression_algo::CompressionStrategy;
pub use compression_algo_resolver::CompressionAlgoResolver;
pub use compression_error::CompressionError;
pub use compression_type::CompressionType;
pub use compression_type::CompressionTypeParseError;
