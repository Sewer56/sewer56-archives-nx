use super::{CompressionResult, DecompressionResult, NxCompressionError, NxDecompressionError};
use crate::api::enums::compression_preference::CompressionPreference;

/// Represents an error specific to LZ4 compression operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Lz4CompressionError {
    /// Compression has failed.
    /// LZ4 doesn't provide an error code here, just returns 0
    CompressionFailed,
}

/// Represents an error specific to LZ4 compression operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Lz4DecompressionError {
    /// Decompression has failed.
    /// LZ4 doesn't provide an error code here, just returns 0
    DecompressionFailed,
}

impl From<Lz4CompressionError> for NxCompressionError {
    fn from(err: Lz4CompressionError) -> NxCompressionError {
        NxCompressionError::Lz4(err)
    }
}

impl From<Lz4DecompressionError> for NxDecompressionError {
    fn from(err: Lz4DecompressionError) -> NxDecompressionError {
        NxDecompressionError::Lz4(err)
    }
}

/// Determines maximum memory needed to alloc to compress data with LZ4.
///
/// # Parameters
///
/// * `source_length`: Number of bytes at source.
pub fn max_alloc_for_compress_size(source_length: usize) -> usize {
    lzzzz::lz4::max_compressed_size(source_length)
}

/// Compresses data with LZ4.
///
/// # Parameters
///
/// * `level`: Level at which we are compressing.
/// * `source`: Source data to compress.
/// * `destination`: Destination buffer for compressed data.
/// * `used_copy`: If this is true, Copy compression was used, due to uncompressible data.
///
/// # Returns
///
/// The number of bytes written to the destination, or an error.
pub fn compress(
    level: i32,
    source: &[u8],
    destination: &mut [u8],
    used_copy: &mut bool,
) -> CompressionResult {
    *used_copy = false;

    let bytes = lzzzz::lz4_hc::compress(source, destination, level);

    if bytes.is_err() {
        // LZ4 only has 1 error
        return Err(Lz4CompressionError::CompressionFailed.into());
    }

    // Note: This code assumes that the user has properly used max_alloc_for_compress_size
    //       failure to do so will result in possible CompressionFailed error.
    let num_bytes = unsafe { bytes.unwrap_unchecked() } as usize;
    if unsafe { bytes.unwrap_unchecked() } > source.len() {
        return super::compress(
            CompressionPreference::Copy,
            level,
            source,
            destination,
            used_copy,
        );
    }

    Ok(num_bytes)
}

/// Decompresses data with LZ4.
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
///
/// # Returns
///
/// The number of bytes written to the destination, or an error.
pub fn decompress(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    let result = lzzzz::lz4::decompress(source, destination);

    match result {
        Ok(num_bytes) => Ok(num_bytes),
        // LZ4 only has a single 'decompression failed', so we don't need to check error type.
        Err(_) => Err(Lz4DecompressionError::DecompressionFailed.into()),
    }
}

/// Partially decompresses data with LZ4 until the destination buffer is filled.
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
///
/// # Returns
///
/// The number of bytes written to the destination, or an error.
pub fn decompress_partial(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    let result = lzzzz::lz4::decompress_partial(source, destination, destination.len());

    match result {
        Ok(num_bytes) => Ok(num_bytes),
        // LZ4 only has a single 'decompression failed', so we don't need to check error type.
        Err(_) => Err(Lz4DecompressionError::DecompressionFailed.into()),
    }
}
