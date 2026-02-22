use super::{CompressionResult, DecompressionResult, R3ACompressionError};
use core::{cmp::min, ptr::copy_nonoverlapping};
use thiserror_no_std::Error;
pub use zstd_sys::ZSTD_ErrorCode;

/// Represents raw errors returned directly from the Copy (no compression) operations.
///
/// This enum contains only errors that originate from copy operations and maintains
/// consistency with other compression algorithm error types. High-level validation
/// errors are handled by [`super::R3ADecompressionError`] variants.
///
/// # Error Mappings
///
/// - `DestinationTooSmall`: Destination buffer is too small to hold the copied data
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum CopyDecompressionError {
    /// Destination buffer too small for copy operation
    #[error("Destination buffer too small for copy operation")]
    DestinationTooSmall,
}

/// Determines maximum memory needed to alloc to compress data with copying.
pub fn max_alloc_for_compress_size(source_length: usize) -> usize {
    source_length
}

/// Compresses data with Copy
///
/// # Parameters
///
/// * `method`: Method we compress with.
/// * `source`: Length of the source in bytes.
/// * `destination`: Pointer to destination.
/// * `used_copy`: If this is true, data was uncompressible and default compression was used instead.
pub fn compress(source: &[u8], destination: &mut [u8], used_copy: &mut bool) -> CompressionResult {
    // Check if destination too small
    if destination.len() < source.len() {
        return Err(R3ACompressionError::DestinationTooSmall);
    }

    *used_copy = true;
    unsafe { copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr(), source.len()) };
    Ok(source.len())
}

/// Decompresses data with Copy
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
pub fn decompress(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    // Check if destination too small
    if destination.len() < source.len() {
        return Err(CopyDecompressionError::DestinationTooSmall.into());
    }

    unsafe { copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr(), source.len()) };
    Ok(source.len())
}

/// Partially decompresses (copies) data until the destination buffer is filled
///
/// # Parameters
///
/// * `source`: Source data to decompress (copy).
/// * `destination`: Destination buffer for decompressed data.
/// * `max_block_size`: Maximum block size for decompression. Ignored for copy algorithm.
pub fn decompress_partial(
    source: &[u8],
    destination: &mut [u8],
    _max_block_size: usize,
) -> DecompressionResult {
    let copy_length = min(source.len(), destination.len());

    unsafe { copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr(), copy_length) };
    Ok(copy_length)
}
