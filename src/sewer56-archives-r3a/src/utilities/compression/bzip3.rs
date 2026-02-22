use core::{
    cmp::{max, min},
    hint::unreachable_unchecked,
    ptr::copy_nonoverlapping,
};

use super::{
    copy, CompressionResult, DecompressionResult, R3ACompressionError, R3ADecompressionError,
};
use crate::prelude::*;
use derive_more::derive::{Deref, DerefMut};
use derive_new::new;
use libbzip3_sys::*;
use thiserror_no_std::Error;

pub const MIN_BLOCK_SIZE: usize = 65 * 1024; // 65 KiB
pub const MAX_BLOCK_SIZE: usize = 511 * 1024 * 1024; // 511 MiB

/// Represents raw errors returned directly from the BZip3 library.
///
/// This enum contains only errors that originate from the underlying libbzip3-sys
/// library and are passed through without interpretation. High-level validation
/// errors are handled by [`R3ACompressionError`] and [`R3ADecompressionError`] variants.
///
/// # Error Code Mappings
///
/// Each variant corresponds to a specific error code from the BZip3 library:
/// - `OutOfBounds` → `BZ3_ERR_OUT_OF_BOUNDS`: Data index out of bounds
/// - `BwtFailed` → `BZ3_ERR_BWT`: Burrows-Wheeler transform failed  
/// - `CrcFailed` → `BZ3_ERR_CRC`: CRC32 check failed
/// - `MalformedHeader` → `BZ3_ERR_MALFORMED_HEADER`: Malformed header detected
/// - `TruncatedData` → `BZ3_ERR_TRUNCATED_DATA`: Data was truncated
/// - `DataTooLarge` → `BZ3_ERR_DATA_TOO_BIG`: Data too large for processing
/// - `InitFailed` → `BZ3_ERR_INIT`: Initialization failed
/// - `DataSizeTooSmall` → `BZ3_ERR_DATA_SIZE_TOO_SMALL`: Buffer size too small for block decoder
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum Bzip3CompressionError {
    /// Out of bounds error occurred during compression
    #[error("Data index out of bounds")]
    OutOfBounds,
    /// BWT transform failed
    #[error("Burrows-Wheeler transform failed")]
    BwtFailed,
    /// CRC check failed
    #[error("CRC32 check failed")]
    CrcFailed,
    /// Malformed header detected
    #[error("Malformed header")]
    MalformedHeader,
    /// Data was truncated
    #[error("Truncated data")]
    TruncatedData,
    /// Data too large for processing
    #[error("Too much data")]
    DataTooLarge,
    /// Initialization failed
    #[error("Initialization failed")]
    InitFailed,
    /// Data size too small for processing
    #[error("Size of buffer `buffer_size` passed to the block decoder (bz3_decode_block) is too small. See function docs for details.")]
    DataSizeTooSmall,
}

/// Represents an error specific to BZip3 decompression operations.
pub type Bzip3DecompressionError = Bzip3CompressionError;

/// SafeState wrapper to ensure proper cleanup of BZip3 state
#[derive(Deref, DerefMut, new)]
pub struct SafeState(*mut bz3_state);

impl Drop for SafeState {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { bz3_free(self.0) };
        }
    }
}

/// Convert BZip3 error code to Result
fn convert_error(error_code: i32) -> Bzip3CompressionError {
    match error_code {
        0 => unsafe { unreachable_unchecked() }, // BZ3_OK
        BZ3_ERR_OUT_OF_BOUNDS => Bzip3CompressionError::OutOfBounds,
        BZ3_ERR_BWT => Bzip3CompressionError::BwtFailed,
        BZ3_ERR_CRC => Bzip3CompressionError::CrcFailed,
        BZ3_ERR_MALFORMED_HEADER => Bzip3CompressionError::MalformedHeader,
        BZ3_ERR_TRUNCATED_DATA => Bzip3CompressionError::TruncatedData,
        BZ3_ERR_DATA_TOO_BIG => Bzip3CompressionError::DataTooLarge,
        BZ3_ERR_INIT => Bzip3CompressionError::InitFailed,
        BZ3_ERR_DATA_SIZE_TOO_SMALL => Bzip3CompressionError::DataSizeTooSmall,
        _ => Bzip3CompressionError::InitFailed, // Default to init failed for unknown errors
    }
}

/// Determines maximum memory needed to allocate to compress data with BZip3.
///
/// # Parameters
///
/// * `source_length`: Number of bytes at source.
pub fn max_alloc_for_compress_size(source_length: usize) -> usize {
    unsafe { bz3_bound(source_length) }
}

/// Compresses data with BZip3.
///
/// # Parameters
///
/// * `source`: Source data to compress.
/// * `destination`: Destination buffer for compressed data.
/// * `used_copy`: If this is true, Copy compression was used, due to uncompressible data.
///
/// # Returns
///
/// The number of bytes written to the destination, or an error.
pub fn compress(source: &[u8], destination: &mut [u8], used_copy: &mut bool) -> CompressionResult {
    *used_copy = false;

    // If destination is too small, return high-level validation error.
    // For bz3, we need to copy the source into destination, and bz3 will work on the destination directly.
    // Therefore, if our destination is too short, we need to guard against user error here.
    if destination.len() < max_alloc_for_compress_size(source.len()) {
        return Err(R3ACompressionError::DestinationTooSmall);
    }

    // bzip3 has a min block size of 65K
    let block_size = max(source.len(), MIN_BLOCK_SIZE) as i32;

    // Create new BZip3 state
    let state = unsafe { SafeState::new(bz3_new(block_size)) };
    if state.is_null() {
        return Err(Bzip3CompressionError::InitFailed.into());
    }

    // Encode single block
    unsafe { copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr(), source.len()) };
    let result = unsafe { bz3_encode_block(*state, destination.as_mut_ptr(), source.len() as i32) };

    if result <= 0 {
        // Get specific error from state
        let error_code = unsafe { bz3_last_error(*state) };
        return Err(convert_error(error_code as i32).into());
    }

    // If compressed size is larger than original, use copy compression
    if result as usize > source.len() {
        return copy::compress(source, destination, used_copy);
    }

    Ok(result as usize)
}

/// Compresses data using BZip3 with early termination support.
/// Note: BZip3 does not have a true streaming API, so this delegates to the regular compress method.
///
/// # Parameters
///
/// * `source`: Source data to compress.
/// * `destination`: Destination buffer for compressed data.
/// * `terminate_early`: Optional callback that returns `Some(usize)` to terminate early
///   with that value, or `None` to continue compression.
/// * `used_copy`: If this is true, Copy compression was used, due to uncompressible data.
///
/// # Returns
///
/// * `Ok(usize)`: The number of bytes written to the destination.
/// * `Err(R3ACompressionError)`: If compression fails.
pub fn compress_streamed<F>(
    source: &[u8],
    destination: &mut [u8],
    terminate_early: Option<F>,
    used_copy: &mut bool,
) -> CompressionResult
where
    F: Fn() -> Option<usize>,
{
    // Check for early termination before starting
    if let Some(ref callback) = terminate_early {
        if let Some(early_result) = callback() {
            return Err(R3ACompressionError::TerminatedStream(early_result));
        }
    }

    // Delegate to regular compress since BZip3 doesn't have streaming
    compress(source, destination, used_copy)
}

/// Decompresses data with BZip3.
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data. Should have length of the data inside.
///
/// # Returns
///
/// The number of bytes written to the destination, or an error.
pub fn decompress(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    // bzip3 has a min block size of 65K
    let block_size = max(destination.len(), MIN_BLOCK_SIZE) as i32;

    // Create new BZip3 state
    let state = unsafe { SafeState::new(bz3_new(block_size)) };
    if state.is_null() {
        return Err(Bzip3DecompressionError::InitFailed.into());
    }

    // Attempt 1: Try direct decompression into destination buffer
    //            This is possible *most* of the time, except some very rare cases.
    //            https://github.com/iczelia/bzip3/pull/144/files#diff-e89cf2cf0812ad6cc411e32e39cd14f8a9fcbb5ff29abcdaff537949e6583164
    //
    // Namely:
    //      Note(sewer): It's technically valid within the spec to create a bzip3 block
    //      where the size after LZP/RLE is larger than the original input. Some earlier encoders
    //      even (mistakenly?) were able to do this; and that's the edge case here where `bz3_decode_block`
    //      might fail.
    //
    // Note: R3A library itself will never produce cases where `destination >= source`, but
    //       handcrafted malicious archives might.
    //       Therefore, because it's 'impossible', we explicitly decide not to handle the buggy case here;
    //       instead we error.
    if destination.len() >= source.len() {
        // Copy source data into destination buffer for direct decompression attempt
        unsafe {
            copy_nonoverlapping(source.as_ptr(), destination.as_mut_ptr(), source.len());
        }

        // Attempt direct decompression
        let result = unsafe {
            bz3_decode_block(
                *state,
                destination.as_mut_ptr(),
                destination.len(),
                source.len() as i32,
                destination.len() as i32,
            )
        };

        if result > 0 {
            // Direct decompression succeeded
            return Ok(result as usize);
        }

        // Check if the error is specifically DataSizeTooSmall
        let error_code = unsafe { bz3_last_error(*state) };
        return Err(convert_error(error_code as i32).into());
    }

    // Note: Past buggy encoder is unsupported.
    Err(Bzip3DecompressionError::DataSizeTooSmall.into())
}

/// Partially decompresses data with BZip3.
/// Note: BZip3 does not support partial decompression, so this delegates to the regular decompress
/// method. If the buffer is too small, this will fail.
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
/// * `max_block_size`: Maximum block size for decompression. Used to allocate temporary buffer for full decompression.
///
/// # Returns
///
/// The number of bytes written to the destination, or an error.
pub fn decompress_partial(
    source: &[u8],
    destination: &mut [u8],
    max_block_size: usize,
) -> DecompressionResult {
    // Validate max_block_size parameter
    if max_block_size == 0 {
        return Err(R3ADecompressionError::MaxBlockSizeNotProvided);
    }

    if max_block_size < destination.len() {
        return Err(R3ADecompressionError::MaxBlockSizeTooSmall);
    }

    // Allocate temporary buffer using max_block_size for full decompression
    let mut temp_buffer = unsafe { Box::new_uninit_slice(max_block_size).assume_init() };

    // Decompress into temporary buffer
    let decompressed_size = decompress(source, &mut temp_buffer)?;

    // Copy only the portion that fits into the destination buffer
    let copy_len = min(decompressed_size, destination.len());

    unsafe {
        copy_nonoverlapping(temp_buffer.as_ptr(), destination.as_mut_ptr(), copy_len);
    }

    Ok(copy_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utilities::compression::R3ADecompressionError;

    #[cfg(feature = "nightly")]
    pub use alloc::vec;
    #[cfg(not(feature = "nightly"))]
    pub use allocator_api2::vec;

    #[test]
    fn decompress_partial_returns_error_when_max_block_size_not_provided() {
        // Create some dummy compressed data (we won't get far enough to decompress it)
        let compressed_data = vec![0u8; 100];
        let mut destination = vec![0u8; 50];

        let result = decompress_partial(&compressed_data, &mut destination, 0);

        assert!(
            result.is_err(),
            "Should return an error when max_block_size is 0"
        );
        match result.unwrap_err() {
            R3ADecompressionError::MaxBlockSizeNotProvided => {
                // Expected error
            }
            _ => panic!("Expected MaxBlockSizeNotProvided error"),
        }
    }

    #[test]
    fn decompress_partial_returns_error_when_max_block_size_too_small() {
        // Create some dummy compressed data (we won't get far enough to decompress it)
        let compressed_data = vec![0u8; 100];
        let mut destination = vec![0u8; 50];
        let max_block_size = 25; // Smaller than destination buffer

        let result = decompress_partial(&compressed_data, &mut destination, max_block_size);

        assert!(
            result.is_err(),
            "Should return an error when max_block_size < destination.len()"
        );
        match result.unwrap_err() {
            R3ADecompressionError::MaxBlockSizeTooSmall => {
                // Expected error
            }
            _ => panic!("Expected MaxBlockSizeTooSmall error"),
        }
    }

    #[test]
    fn decompress_partial_returns_error_when_max_block_size_insufficient_for_actual_data() {
        // Create test data and compress it
        let test_data = b"This is test data for BZip3 compression that should be longer than our max_block_size but shorter than destination";
        let mut compressed = vec![0u8; super::max_alloc_for_compress_size(test_data.len())];
        let mut used_copy = false;

        let compressed_size = super::super::compress(
            super::super::CompressionPreference::Bzip3,
            0,
            test_data,
            &mut compressed,
            &mut used_copy,
        )
        .unwrap();
        compressed.truncate(compressed_size);

        // Create destination buffer smaller than test_data
        let mut destination = vec![0u8; test_data.len() / 2];
        // Set max_block_size larger than destination but smaller than actual decompressed size
        let max_block_size = destination.len() + 10; // Larger than destination but insufficient

        let result = decompress_partial(&compressed, &mut destination, max_block_size);

        assert!(
            result.is_err(),
            "Should return an error when max_block_size is insufficient for actual decompressed data"
        );

        // Should get a BZip3 decompression error, not our validation errors
        match result.unwrap_err() {
            R3ADecompressionError::MaxBlockSizeNotProvided
            | R3ADecompressionError::MaxBlockSizeTooSmall => {
                panic!("Should not get validation errors when max_block_size > destination.len()");
            }
            R3ADecompressionError::Bzip3(_) => {
                // Expected - some BZip3 decompression error due to insufficient buffer
            }
            _ => panic!("Expected BZip3 decompression error"),
        }
    }
}
