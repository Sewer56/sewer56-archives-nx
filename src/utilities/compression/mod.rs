// Compression modules
pub mod copy;

#[cfg(feature = "zstd")]
pub mod zstd;

#[cfg(feature = "lz4")]
pub mod lz4;

use crate::api::enums::compression_preference::CompressionPreference;
use copy::*;
use lz4::{Lz4CompressionError, Lz4DecompressionError};

/// A result type around compression functions..
/// Either a success code (number of bytes written), or an error code.
pub type CompressionResult = Result<usize, NxCompressionError>;

/// Represents an error returned from the Nx compression APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NxCompressionError {
    Copy(CopyCompressionError),
    ZStandard(ZSTD_ErrorCode),
    Lz4(Lz4CompressionError),
}

/// A result type around compression functions..
/// Either a success code (number of bytes decompressed), or an error code.
pub type DecompressionResult = Result<usize, NxDecompressionError>;

/// Represents an error returned from the Nx compression APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NxDecompressionError {
    Copy(CopyDecompressionError),
    ZStandard(ZSTD_ErrorCode),
    Lz4(Lz4DecompressionError),
}

/// Determines maximum memory needed to alloc to compress data with any method.
///
/// # Parameters
///
/// * `source_length`: Number of bytes at source.
pub fn max_alloc_for_compress_size(source_length: usize) -> usize {
    let mut max_size = copy::max_alloc_for_compress_size(source_length);
    #[cfg(feature = "lz4")]
    {
        max_size = lz4::max_alloc_for_compress_size(source_length).max(max_size)
    }
    #[cfg(feature = "zstd")]
    {
        max_size = zstd::max_alloc_for_compress_size(source_length).max(max_size)
    }
    max_size
}

/// Determines memory needed to alloc to compress data with a specified method.
///
/// # Parameters
///
/// * `method`: Method we compress with.
/// * `source_length`: Number of bytes at source.
pub fn alloc_for_compress_size(method: CompressionPreference, source_length: usize) -> usize {
    match method {
        CompressionPreference::Lz4 => lz4::max_alloc_for_compress_size(source_length),
        CompressionPreference::ZStandard => zstd::max_alloc_for_compress_size(source_length),
        CompressionPreference::Copy => copy::max_alloc_for_compress_size(source_length),
        _ => unimplemented!(),
    }
}

/// Compresses data with a specific method.
///
/// # Parameters
///
/// * `method`: Method we compress with.
/// * `level`: Level at which we are compressing.
/// * `source`: Source data to compress.
/// * `destination`: Destination buffer for compressed data.
/// * `used_copy`: If this is true, Copy compression was used, due to uncompressible data or by request.
///
/// # Returns
///
/// The number of bytes written to the destination.
pub fn compress(
    method: CompressionPreference,
    level: i32,
    source: &[u8],
    destination: &mut [u8],
    used_copy: &mut bool,
) -> CompressionResult {
    *used_copy = false;
    match method {
        CompressionPreference::Copy => copy::compress(source, destination, used_copy),
        #[cfg(feature = "zstd")]
        CompressionPreference::ZStandard => zstd::compress(level, source, destination, used_copy),
        #[cfg(feature = "lz4")]
        CompressionPreference::Lz4 => lz4::compress(level, source, destination, used_copy),
        _ => panic!("Unsupported compression method"),
    }
}

/// Decompresses data with a specific method.
///
/// # Parameters
///
/// * `method`: Method we decompress with.
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
pub fn decompress(
    method: CompressionPreference,
    source: &[u8],
    destination: &mut [u8],
) -> DecompressionResult {
    match method {
        CompressionPreference::Copy => copy::decompress(source, destination),
        #[cfg(feature = "zstd")]
        CompressionPreference::ZStandard => zstd::decompress(source, destination),
        #[cfg(feature = "lz4")]
        CompressionPreference::Lz4 => lz4::decompress(source, destination),
        _ => panic!("Unsupported decompression method"),
    }
}
