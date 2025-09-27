use super::{CompressionResult, DecompressionResult, NxCompressionError, NxDecompressionError};
use core::{cmp::min, ptr::copy_nonoverlapping};
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
    // Check if destination too small
    if destination.len() < source.len() {
        return Err(NxCompressionError::DestinationTooSmall);
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
        return Err(NxDecompressionError::DestinationTooSmall);
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
