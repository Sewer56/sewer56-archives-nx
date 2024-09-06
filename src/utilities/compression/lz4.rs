use super::{CompressionResult, DecompressionResult, NxCompressionError, NxDecompressionError};
use crate::api::enums::compression_preference::CompressionPreference;
use lz4_sys::*;

/// Represents an error specific to LZ4 compression operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lz4CompressionError {
    /// Compression has failed.
    /// LZ4 doesn't provide an error code here, just returns 0
    CompressionFailed,
}

/// Represents an error specific to LZ4 compression operations.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    unsafe { LZ4_compressBound(source_length as i32) as usize }
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

    let bytes = unsafe {
        LZ4_compress_default(
            source.as_ptr() as *const i8,
            destination.as_mut_ptr() as *mut i8,
            source.len() as i32,
            destination.len() as i32,
        )
    };

    if bytes <= 0 {
        return Err(Lz4CompressionError::CompressionFailed.into());
    }
    if bytes as usize > source.len() {
        return super::compress(
            CompressionPreference::Copy,
            level,
            source,
            destination,
            used_copy,
        );
    }

    Ok(bytes as usize)
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
    let source_length = source.len();
    let destination_length = destination.len();

    let result = unsafe {
        LZ4_decompress_safe(
            source.as_ptr() as *const i8,
            destination.as_mut_ptr() as *mut i8,
            source_length as i32,
            destination_length as i32,
        )
    };

    if result < 0 {
        Err(Lz4DecompressionError::DecompressionFailed.into())
    } else {
        Ok(result as usize)
    }
}
