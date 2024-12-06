use core::cmp::min;

use super::{CompressionResult, DecompressionResult};
use crate::api::enums::*;
use lz4_sys::*;
use thiserror_no_std::Error;

/// Represents an error specific to LZ4 compression operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum Lz4CompressionError {
    /// Compression has failed.
    /// LZ4 doesn't provide an error code here, just returns 0
    #[error("LZ4 Compression Failed")]
    CompressionFailed,
}

/// Represents an error specific to LZ4 compression operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum Lz4DecompressionError {
    /// Decompression has failed.
    /// LZ4 doesn't provide an error code here, just returns 0
    #[error("LZ4 Decompression Failed")]
    DecompressionFailed,
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

    let result = unsafe {
        LZ4_compress_HC(
            source.as_ptr() as *const c_char,
            destination.as_mut_ptr() as *mut c_char,
            source.len() as c_int,
            destination.len() as c_int,
            level as c_int,
        )
    };

    if result == 0 {
        // LZ4 only has 1 error
        return Err(Lz4CompressionError::CompressionFailed.into());
    }

    // Note: This code assumes that the user has properly used max_alloc_for_compress_size
    //       failure to do so will result in possible CompressionFailed error.
    if result > source.len() as i32 {
        return super::compress(
            CompressionPreference::Copy,
            level,
            source,
            destination,
            used_copy,
        );
    }

    Ok(result as usize)
}

/// Compresses data using streaming compression with LZ4-HC.
///
/// This function allows compression of data in chunks while providing the ability
/// to terminate compression early through a callback function. Data is processed
/// in blocks of 128KB.
///
/// # Parameters
///
/// * `level`: Compression level to use.
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
///
/// # Safety
///
/// This function uses unsafe code to interact with the LZ4 C API.
pub fn compress_streamed<F>(
    level: i32,
    source: &[u8],
    destination: &mut [u8],
    terminate_early: Option<F>,
    used_copy: &mut bool,
) -> CompressionResult
where
    F: Fn() -> Option<i32>,
{
    *used_copy = false;
    const BLOCK_SIZE: usize = 131072;

    unsafe {
        // Create LZ4 HC stream
        let stream = LZ4_createStreamHC();
        if stream.is_null() {
            return Err(Lz4CompressionError::CompressionFailed.into());
        }

        // Set compression level
        LZ4_setCompressionLevel(stream, level);

        let result = {
            let mut total_written = 0;
            let mut total_read = 0;
            let source_len = source.len();

            while total_read < source_len {
                // Calculate chunk size for this iteration
                let remaining = source_len - total_read;
                let chunk_size = min(BLOCK_SIZE, remaining);

                // Compress the current chunk
                let num_compressed = LZ4_compress_HC_continue(
                    stream,
                    // SAFETY: total_read is less than source_len, guaranteed by loop above.
                    source.as_ptr().add(total_read) as *const c_char,
                    destination.as_mut_ptr().add(total_written) as *mut c_char,
                    chunk_size as c_int,
                    (destination.len() - total_written) as c_int,
                );

                if num_compressed <= 0 {
                    LZ4_freeStreamHC(stream);
                    return Err(Lz4CompressionError::CompressionFailed.into());
                }

                // Check for early termination
                if let Some(ref callback) = terminate_early {
                    if let Some(early_result) = callback() {
                        LZ4_freeStreamHC(stream);
                        return Ok(early_result as usize);
                    }
                }

                // Check if compression is efficient for this chunk
                if num_compressed as usize > chunk_size {
                    LZ4_freeStreamHC(stream);
                    return super::compress(
                        super::CompressionPreference::Copy,
                        level,
                        source,
                        destination,
                        used_copy,
                    );
                }

                total_written += num_compressed as usize;
                total_read += chunk_size;
            }

            Ok(total_written)
        };

        // Clean up
        LZ4_freeStreamHC(stream);
        result
    }
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
    let result = unsafe {
        LZ4_decompress_safe(
            source.as_ptr() as *const c_char,
            destination.as_mut_ptr() as *mut c_char,
            source.len() as c_int,
            destination.len() as c_int,
        )
    };

    if result <= 0 {
        // LZ4 only has a single 'decompression failed', so we don't need to check error type.
        Err(Lz4DecompressionError::DecompressionFailed.into())
    } else {
        Ok(result as usize)
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
    let result = unsafe {
        LZ4_decompress_safe_partial(
            source.as_ptr() as *const c_char,
            destination.as_mut_ptr() as *mut c_char,
            source.len() as c_int,
            destination.len() as c_int,
            destination.len() as c_int,
        )
    };

    if result <= 0 {
        // LZ4 only has a single 'decompression failed', so we don't need to check error type.
        Err(Lz4DecompressionError::DecompressionFailed.into())
    } else {
        Ok(result as usize)
    }
}

// Missing bindings from lz4-sys declared here.
extern "C" {
    #[allow(non_snake_case)]
    pub fn LZ4_decompress_safe_partial(
        source: *const c_char,
        dest: *mut c_char,
        sourceSize: c_int,
        targetOutputSize: c_int,
        maxDestSize: c_int,
    ) -> c_int;

    #[allow(non_snake_case)]
    pub fn LZ4_createStreamHC() -> *mut LZ4StreamEncode;

    #[allow(non_snake_case)]
    pub fn LZ4_freeStreamHC(ptr: *mut LZ4StreamEncode) -> c_int;

    #[allow(non_snake_case)]
    pub fn LZ4_setCompressionLevel(ptr: *mut LZ4StreamEncode, compression_level: c_int);

    #[allow(non_snake_case)]
    pub fn LZ4_compress_HC_continue(
        ptr: *mut LZ4StreamEncode,
        src: *const c_char,
        dst: *mut c_char,
        src_size: c_int,
        dst_capacity: c_int,
    ) -> c_int;
}

#[cfg(test)]
mod tests {
    use super::super::max_alloc_for_compress_size;
    use super::*;
    use alloc::vec;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn compress_streamed_basic() {
        let original_data = b"Hello, LZ4!".repeat(100_000); // Large enough to test multiple chunks
        let mut compressed = vec![0u8; max_alloc_for_compress_size(original_data.len())];
        let mut used_copy = false;

        let result = compress_streamed(
            3,
            &original_data,
            &mut compressed,
            None as Option<fn() -> Option<i32>>,
            &mut used_copy,
        )
        .unwrap();

        assert!(!used_copy, "Should not have used copy compression");
        assert!(result > 0, "Should have compressed some data");
        assert!(
            result < original_data.len(),
            "Should achieve some compression"
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn compress_streamed_early_termination() {
        let original_data = b"Hello, LZ4!".repeat(100_000);
        let mut compressed = vec![0u8; max_alloc_for_compress_size(original_data.len())];
        let mut used_copy = false;

        let early_return_value = 42;
        let result = compress_streamed(
            3,
            &original_data,
            &mut compressed,
            Some(|| Some(early_return_value)),
            &mut used_copy,
        )
        .unwrap();

        assert_eq!(
            result, early_return_value as usize,
            "Should return early with callback value"
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn compress_streamed_small_destination() {
        let original_data = b"Hello, LZ4!".repeat(100_000);
        let mut compressed = vec![0u8; 10]; // Too small destination buffer
        let mut used_copy = false;

        let result = compress_streamed(
            3,
            &original_data,
            &mut compressed,
            None as Option<fn() -> Option<i32>>,
            &mut used_copy,
        );

        assert!(result.is_err(), "Should fail");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn compress_streamed_multiple_chunks() {
        let original_data = b"Hello, LZ4!".repeat(200_000); // Ensures multiple chunks
        let mut compressed = vec![0u8; max_alloc_for_compress_size(original_data.len())];
        let mut used_copy = false;

        let result = compress_streamed(
            3,
            &original_data,
            &mut compressed,
            None as Option<fn() -> Option<i32>>,
            &mut used_copy,
        )
        .unwrap();

        assert!(!used_copy, "Should not have used copy compression");
        assert!(result > 0, "Should have compressed some data");
        assert!(
            result < original_data.len(),
            "Should achieve some compression"
        );
    }
}
