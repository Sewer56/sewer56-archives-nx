use core::{cmp::max, hint::unreachable_unchecked, ptr::copy_nonoverlapping};

use super::{copy, CompressionResult, DecompressionResult};
use crate::prelude::*;
use crate::utilities::compression::NxCompressionError;
use derive_more::derive::{Deref, DerefMut};
use derive_new::new;
use libbzip3_sys::*;
use thiserror_no_std::Error;

pub const MIN_BLOCK_SIZE: usize = 65 * 1024; // 65 KiB
pub const MAX_BLOCK_SIZE: usize = 511 * 1024 * 1024; // 511 MiB

/// Represents an error specific to BZip3 compression operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum Bzip3CompressionError {
    /// Out of bounds error occurred during compression
    #[error("BZip3 Out of Bounds Error")]
    OutOfBounds,
    /// BWT transform failed
    #[error("BZip3 BWT Transform Failed")]
    BwtFailed,
    /// CRC check failed
    #[error("BZip3 CRC Check Failed")]
    CrcFailed,
    /// Malformed header detected
    #[error("BZip3 Malformed Header")]
    MalformedHeader,
    /// Data was truncated
    #[error("BZip3 Truncated Data")]
    TruncatedData,
    /// Data too large for processing
    #[error("BZip3 Data Too Large")]
    DataTooLarge,
    /// Initialization failed
    #[error("BZip3 Initialization Failed")]
    InitFailed,
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
        -1 => Bzip3CompressionError::OutOfBounds,
        -2 => Bzip3CompressionError::BwtFailed,
        -3 => Bzip3CompressionError::CrcFailed,
        -4 => Bzip3CompressionError::MalformedHeader,
        -5 => Bzip3CompressionError::TruncatedData,
        -6 => Bzip3CompressionError::DataTooLarge,
        -7 => Bzip3CompressionError::InitFailed,
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

    // bzip3 has a max block size of 512MiB
    // if we issue 512MiB blocks, that will fail.
    if source.len() > MAX_BLOCK_SIZE {
        return Err(Bzip3CompressionError::DataTooLarge.into());
    }

    // If destination is too small, defer back to copy.
    // bz3 doesn't do that check, so we need to do it.
    if destination.len() < max_alloc_for_compress_size(source.len()) {
        return copy::compress(source, destination, used_copy);
    }

    // bzip3 has a min block size of 64K
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
/// * `terminate_early`: Optional callback that returns `Some(i32)` to terminate early
///   with that value, or `None` to continue compression.
/// * `used_copy`: If this is true, Copy compression was used, due to uncompressible data.
///
/// # Returns
///
/// * `Ok(usize)`: The number of bytes written to the destination.
/// * `Err(NxCompressionError)`: If compression fails.
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
            return Err(NxCompressionError::TerminatedStream(early_result));
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
    // bzip3 has a max block size of 512MiB
    // if we issue 512MiB blocks, that will fail.
    if destination.len() > MAX_BLOCK_SIZE {
        return Err(Bzip3CompressionError::DataTooLarge.into());
    }

    // bzip3 has a min block size of 64K
    let block_size = max(destination.len(), MIN_BLOCK_SIZE) as i32;

    // Create new BZip3 state
    let state = unsafe { SafeState::new(bz3_new(block_size)) };
    if state.is_null() {
        return Err(Bzip3DecompressionError::InitFailed.into());
    }

    // TODO: This is inefficient, but in some cases, it's not possible to give more bytes to destination
    //       when calling this, for example when working with memory mapped files.
    // It's not documented currently, but destination in bz3 needs to be bounded
    let dest_num_bytes = unsafe { bz3_bound(destination.len()) };
    let mut decomp_destination = unsafe { Box::new_uninit_slice(dest_num_bytes).assume_init() };

    // Decode single block
    let result = unsafe {
        // SAFETY: Program will always use bound call before this
        copy_nonoverlapping(
            source.as_ptr(),
            decomp_destination.as_mut_ptr(),
            source.len(),
        );

        bz3_decode_block(
            *state,
            decomp_destination.as_mut_ptr(),
            source.len() as i32,
            destination.len() as i32,
        )
    };

    if result <= 0 {
        // Get specific error from state
        let error_code = unsafe { bz3_last_error(*state) };
        return Err(convert_error(error_code as i32).into());
    }

    // Copy decompressed data to original destination
    unsafe {
        copy_nonoverlapping(
            decomp_destination.as_ptr(),
            destination.as_mut_ptr(),
            result as usize,
        )
    };

    Ok(result as usize)
}

/// Partially decompresses data with BZip3.
/// Note: BZip3 does not support partial decompression, so this delegates to the regular decompress
/// method. If the buffer is too small, this will fail.
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
    // Partial decompression is not supported in bzip3, we must emulate it by
    // allocating a new buffer.

    decompress(source, destination)
}
