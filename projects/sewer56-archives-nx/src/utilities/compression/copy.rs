use super::{CompressionResult, DecompressionResult};
use core::{cmp::min, ptr::copy_nonoverlapping};
use thiserror_no_std::Error;
pub use zstd_sys::ZSTD_ErrorCode;

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
    // Check if destination too small in debug mode only
    // In debug builds only
    #[cfg(debug_assertions)]
    if destination.len() < source.len() {
        return Err(CopyCompressionError::DestinationTooSmall.into());
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
    // Check if destination too small in debug mode only
    // In debug builds only
    #[cfg(debug_assertions)]
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
pub fn decompress_partial(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    let copy_length = min(source.len(), destination.len());

    unsafe { copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr(), copy_length) };
    Ok(copy_length)
}

/// An error occurred during copy compression.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum CopyCompressionError {
    #[error("Destination buffer too small for copy compress")]
    DestinationTooSmall,
}

/// An error occurred during copy decompression.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum CopyDecompressionError {
    #[error("Destination buffer too small for copy decompress")]
    DestinationTooSmall,
}
