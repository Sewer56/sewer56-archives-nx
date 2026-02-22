use super::dictionary::ZstdCompressionDict;
use super::{CompressionResult, DecompressionResult, R3ACompressionError, R3ADecompressionError};
use crate::utilities::compression::copy;
use core::cmp::min;
use core::ffi::c_void;
use core::ptr::NonNull;
use derive_more::derive::{Deref, DerefMut};
use derive_new::new;
use zstd_sys::ZSTD_ErrorCode::*;
use zstd_sys::ZSTD_cParameter::*;
use zstd_sys::ZSTD_dParameter::*;
use zstd_sys::ZSTD_format_e::*;
use zstd_sys::*;

/// Determines maximum file size for output needed to alloc to compress data with ZStandard.
///
/// # Parameters
///
/// * `source_length`: Number of bytes at source.
pub fn max_alloc_for_compress_size(source_length: usize) -> usize {
    unsafe { ZSTD_compressBound(source_length) }
}

/// Compresses data with ZStandard
///
/// # Parameters
///
/// * `level`: Level at which we are compressing.
/// * `source`: Length of the source in bytes.
/// * `destination`: Pointer to destination.
/// * `used_copy`: If this is true, Copy compression was used, due to uncompressible data.
pub fn compress(
    level: i32,
    source: &[u8],
    destination: &mut [u8],
    used_copy: &mut bool,
) -> CompressionResult {
    *used_copy = false;

    // Create a compression context
    let cctx = unsafe { ZSTD_createCCtx() };
    if cctx.is_null() {
        return Err(R3ACompressionError::ZStandard(
            ZSTD_ErrorCode::ZSTD_error_GENERIC,
        ));
    }

    // Set compression parameters (magicless format, no extra headers)
    zstd_setcommoncompressparams(cctx, Some(level));

    // Perform compression
    let result = unsafe {
        ZSTD_compress2(
            cctx,
            destination.as_mut_ptr() as *mut c_void,
            destination.len(),
            source.as_ptr() as *const c_void,
            source.len(),
        )
    };

    // Free the context
    unsafe {
        ZSTD_freeCCtx(cctx);
    }

    let errcode = unsafe { ZSTD_getErrorCode(result) };
    if errcode == ZSTD_error_dstSize_tooSmall {
        return Err(R3ACompressionError::DestinationTooSmall);
    }
    if result > source.len() {
        return copy::compress(source, destination, used_copy);
    }

    if unsafe { ZSTD_isError(result) } == 0 {
        return Ok(result);
    }

    Err(R3ACompressionError::ZStandard(errcode))
}

/// Compresses data using streaming compression with ZStandard.
///
/// This function allows compression of data in chunks while providing the ability
/// to terminate compression early through a callback function.
///
/// # Parameters
///
/// * `level`: Compression level to use.
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
///
/// # Safety
///
/// This function uses unsafe code to interact with the ZStandard C API.
pub fn compress_streamed<F>(
    level: i32,
    source: &[u8],
    destination: &mut [u8],
    terminate_early: Option<F>,
    used_copy: &mut bool,
) -> CompressionResult
where
    F: Fn() -> Option<usize>,
{
    *used_copy = false;

    unsafe {
        // Create compression stream
        let cstream = SafeCStream::new(ZSTD_createCStream());
        if cstream.is_null() {
            return Err(R3ACompressionError::ZStandard(
                ZSTD_ErrorCode::ZSTD_error_memory_allocation,
            ));
        }

        // Set compression parameters (magicless format, no extra headers)
        zstd_setcommoncompressparams(*cstream, Some(level));

        // Initialize the stream
        let init_result = ZSTD_initCStream(*cstream, level);
        if ZSTD_isError(init_result) != 0 {
            return Err(R3ACompressionError::ZStandard(ZSTD_getErrorCode(
                init_result,
            )));
        }

        // Determine optimal input size for optimal compression performance
        // while doing as little work as possible per chunk.
        let in_buffer_size = ZSTD_CStreamInSize();
        let mut total_read: usize = 0;
        let source_len = source.len();

        let mut output = ZSTD_outBuffer {
            dst: destination.as_mut_ptr() as *mut c_void,
            size: destination.len(),
            pos: 0,
        };

        let mut last_out_pos = 0;
        while total_read < source_len {
            let to_read = min(in_buffer_size, source_len - total_read);
            let last_chunk = to_read < in_buffer_size;
            let mode = if last_chunk {
                ZSTD_EndDirective::ZSTD_e_end
            } else {
                ZSTD_EndDirective::ZSTD_e_continue
            };

            let mut input = ZSTD_inBuffer {
                // SAFETY: total_read is guaranteed under < source_len by while condition above.
                src: source.as_ptr().add(total_read) as *const c_void,
                size: to_read,
                pos: 0,
            };

            let mut finished = false;
            while !finished {
                let result = ZSTD_compressStream2(*cstream, &mut output, &mut input, mode);

                // Check if zstd returned an error, or no bytes were compressed in this iteration.
                // If no bytes were compressed, that means destination buffer is too small.
                let has_error = ZSTD_isError(result) != 0;
                let dest_too_small = output.pos == last_out_pos;
                if dest_too_small {
                    return Err(R3ACompressionError::DestinationTooSmall);
                }
                if has_error {
                    return copy::compress(source, destination, used_copy);
                }
                last_out_pos = output.pos;

                // Check for early termination if callback provided
                if let Some(callback) = &terminate_early {
                if let Some(early_result) = callback() {
                    return Err(R3ACompressionError::TerminatedStream(early_result));
                }
                }

                // If we're on the last chunk we're finished when zstd returns 0,
                // which means its consumed all the input AND finished the frame.
                // Otherwise, we're finished when we've consumed all the input.
                // Note: Copied from zstd example.
                finished = if last_chunk {
                    result == 0
                } else {
                    input.pos == input.size
                };
            }

            total_read += input.pos;

            if last_chunk {
                break;
            }
        }

        // Check if compression was beneficial.
        // If it was not, default to copy.
        if output.pos > source_len {
            return copy::compress(source, destination, used_copy);
        }

        Ok(output.pos)
    }
}

/// Compresses data using a ZStandard dictionary.
///
/// This function allows for compression using a pre-trained dictionary to potentially
/// achieve better compression ratios for similar data.
///
/// # Parameters
///
/// * `dict`: The ZStandard compression dictionary to use.
/// * `source`: Source data to compress.
/// * `destination`: Destination buffer for compressed data.
/// * `used_copy`: If this is true, Copy compression was used, due to uncompressible data.
///
/// # Returns
///
/// The number of bytes written to the destination.
pub fn compress_with_dictionary(
    dict: &ZstdCompressionDict,
    source: &[u8],
    destination: &mut [u8],
    used_copy: &mut bool,
) -> CompressionResult {
    unsafe {
        // Create a compression context
        let cctx_ptr = ZSTD_createCCtx();
        if cctx_ptr.is_null() {
            return Err(R3ACompressionError::ZStandard(
                ZSTD_ErrorCode::ZSTD_error_memory_allocation,
            ));
        }

        // Set compression parameters (magicless format, no extra headers)
        zstd_setcommoncompressparams(cctx_ptr, None);

        // Compress using the dictionary
        let result = dict.compress(
            source,
            destination,
            used_copy,
            NonNull::new_unchecked(cctx_ptr),
        );

        // Free the context and return result
        ZSTD_freeCCtx(cctx_ptr);

        result
    }
}

/// Decompresses data with ZStandard
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
pub fn decompress(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    // Create decompression context
    let dctx = unsafe { ZSTD_createDCtx() };
    if dctx.is_null() {
        return Err(R3ADecompressionError::ZStandard(
            ZSTD_ErrorCode::ZSTD_error_GENERIC,
        ));
    }

    // Set decompression parameters to match compression
    zstd_setcommondecompressionparams(dctx);

    // Perform decompression
    let result = unsafe {
        ZSTD_decompressDCtx(
            dctx,
            destination.as_mut_ptr() as *mut c_void,
            destination.len(),
            source.as_ptr() as *const c_void,
            source.len(),
        )
    };

    // Free the context
    unsafe {
        ZSTD_freeDCtx(dctx);
    }

    if unsafe { ZSTD_isError(result) } != 0 {
        let errcode = unsafe { ZSTD_getErrorCode(result) };
        return Err(R3ADecompressionError::ZStandard(errcode));
    }

    Ok(result)
}

/// Partially decompresses data with ZStandard until the destination buffer is filled
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
/// * `max_block_size`: Maximum block size for decompression. Ignored for ZStandard algorithm.
pub fn decompress_partial(
    source: &[u8],
    destination: &mut [u8],
    _max_block_size: usize,
) -> DecompressionResult {
    unsafe {
        let d_stream = ZSTD_createDStream();

        // Set decompression parameters to match compression
        zstd_setcommondecompressionparams(d_stream);

        let mut out_buf = ZSTD_outBuffer {
            dst: destination.as_mut_ptr() as *mut c_void,
            pos: 0,
            size: destination.len(),
        };
        let mut in_buf = ZSTD_inBuffer {
            src: source.as_ptr() as *const c_void,
            pos: 0,
            size: source.len(),
        };

        while out_buf.pos < destination.len() {
            let result = ZSTD_decompressStream(d_stream, &mut out_buf, &mut in_buf);

            // We ran into an error, o no.
            if ZSTD_isError(result) != 0 {
                let error_code = ZSTD_getErrorCode(result);
                ZSTD_freeDStream(d_stream);
                return Err(R3ADecompressionError::ZStandard(error_code));
            }

            if out_buf.pos != out_buf.size {
                continue;
            }

            // To quote the docs:
            // But if `output.pos == output.size`, there might be some data left within internal buffers.,
            // In which case, call ZSTD_decompressStream() again to flush whatever remains in the buffer.
            ZSTD_decompressStream(d_stream, &mut out_buf, &mut in_buf);
        }

        ZSTD_freeDStream(d_stream);
        Ok(out_buf.pos)
    }
}

/// Determines the decompressed size of ZStandard compressed data
///
/// # Parameters
///
/// * `compressed_data`: Slice containing the compressed data
///
/// # Returns
///
/// * `Ok(usize)`: The decompressed size in bytes
/// * `Err(GetDecompressedSizeError)`: If there was an error determining the size
pub fn get_decompressed_size(compressed_data: &[u8]) -> Result<usize, GetDecompressedSizeError> {
    let size = unsafe {
        ZSTD_findDecompressedSize(
            compressed_data.as_ptr() as *const c_void,
            compressed_data.len(),
        ) as i64
    };

    match size {
        size if size == ZSTD_CONTENTSIZE_ERROR as i64 => {
            Err(GetDecompressedSizeError::UnknownErrorOccurred)
        }
        size if size == ZSTD_CONTENTSIZE_UNKNOWN as i64 => {
            Err(GetDecompressedSizeError::SizeCannotBeDetermined)
        }
        x => Ok(x as usize),
    }
}

/// Compresses data with ZStandard.
/// Does not use fallback to 'copy' if compression is ineffective.
///
/// # Parameters
///
/// * `level`: Level at which we are compressing.
/// * `source`: Length of the source in bytes.
/// * `destination`: Pointer to destination.
pub fn force_compress(level: i32, source: &[u8], destination: &mut [u8]) -> CompressionResult {
    // Create a compression context
    let cctx = unsafe { ZSTD_createCCtx() };
    if cctx.is_null() {
        return Err(R3ACompressionError::ZStandard(
            ZSTD_ErrorCode::ZSTD_error_GENERIC,
        ));
    }

    // Set compression parameters (magicless format, no extra headers)
    zstd_setcommoncompressparams(cctx, Some(level));

    // Perform compression
    let result = unsafe {
        ZSTD_compress2(
            cctx,
            destination.as_mut_ptr() as *mut c_void,
            destination.len(),
            source.as_ptr() as *const c_void,
            source.len(),
        )
    };

    // Free the context
    unsafe {
        ZSTD_freeCCtx(cctx);
    }

    if unsafe { ZSTD_isError(result) } == 0 {
        return Ok(result);
    }

    Err(R3ACompressionError::ZStandard(unsafe {
        ZSTD_getErrorCode(result)
    }))
}

#[inline(always)]
fn zstd_setcommoncompressparams(cctx: *mut ZSTD_CCtx_s, level: Option<i32>) {
    unsafe {
        if let Some(lv) = level {
            ZSTD_CCtx_setParameter(cctx, ZSTD_c_compressionLevel, lv);
        }
        ZSTD_CCtx_setParameter(
            cctx,
            ZSTD_c_experimentalParam2, // zstd_c_format
            ZSTD_f_zstd1_magicless as i32,
        );
        ZSTD_CCtx_setParameter(cctx, ZSTD_c_contentSizeFlag, 0);
        ZSTD_CCtx_setParameter(cctx, ZSTD_c_checksumFlag, 0);
        ZSTD_CCtx_setParameter(cctx, ZSTD_c_dictIDFlag, 0);
    }
}

pub(crate) fn zstd_setcommondecompressionparams(dctx: *mut ZSTD_DCtx_s) {
    unsafe {
        ZSTD_DCtx_setParameter(
            dctx,
            ZSTD_d_experimentalParam1, // zstd_d_format
            ZSTD_f_zstd1_magicless as i32,
        );
    };
}

/// Represents an error returned from the R3A compression APIs.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GetDecompressedSizeError {
    /// The size of the compressed payload cannot be determined.
    SizeCannotBeDetermined,

    /// Unknown ZStandard error has occurred.
    UnknownErrorOccurred,
}

#[derive(Deref, DerefMut, new)]
pub struct SafeCStream(*mut ZSTD_CStream);
impl Drop for SafeCStream {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { ZSTD_freeCStream(self.0) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utilities::compression::dictionary::train_dictionary;
    use alloc::vec;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn decompress_invalid_data_returns_error() {
        let invalid_compressed_data = vec![0xFFu8; 100];
        let mut destination = vec![0u8; 1000];

        let result = decompress(&invalid_compressed_data, &mut destination);

        assert!(
            result.is_err(),
            "Should return an error for invalid compressed data"
        );
        match result {
            Err(R3ADecompressionError::ZStandard(error_code)) => {
                assert_eq!(
                    error_code, ZSTD_error_frameParameter_unsupported,
                    "Not a zstandard file"
                );
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn decompress_partial_invalid_data_returns_error() {
        let invalid_compressed_data = vec![0xFFu8; 100];
        let mut destination = vec![0u8; 1000];

        let result = decompress_partial(&invalid_compressed_data, &mut destination, 0);

        assert!(
            result.is_err(),
            "Should return an error for invalid compressed data"
        );
        match result {
            Err(R3ADecompressionError::ZStandard(error_code)) => {
                assert_eq!(
                    error_code, ZSTD_error_frameParameter_unsupported,
                    "Not a zstandard file"
                );
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn get_decompressed_size() {
        // Create some sample data
        let original_data = b"Hello, ZStandard!".repeat(100);

        // Compress the data
        let mut compressed_data = vec![0u8; max_alloc_for_compress_size(original_data.len())];
        let compressed_size = unsafe {
            ZSTD_compress(
                compressed_data.as_mut_ptr() as *mut c_void,
                compressed_data.len(),
                original_data.as_ptr() as *const c_void,
                original_data.len(),
                3,
            )
        };
        compressed_data.truncate(compressed_size);

        // Get the decompressed size
        let decompressed_size = super::get_decompressed_size(&compressed_data).unwrap();

        assert_eq!(
            decompressed_size,
            original_data.len(),
            "Decompressed size should match original data length"
        );

        // Test with invalid data
        let invalid_data = vec![0u8; 100];
        let result = super::get_decompressed_size(&invalid_data);
        assert!(
            result.is_err(),
            "Should return an error for invalid compressed data"
        );
        assert_eq!(
            result.unwrap_err(),
            GetDecompressedSizeError::UnknownErrorOccurred
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)] // unsupported because calls zstd code
    fn can_compress_with_dictionary() {
        // Create a simple dictionary from sample data
        let samples: [&[u8]; 7] = [
            b"this is a test string",
            b"this is another test string",
            b"yet another test string",
            b"one more test string",
            b"fifth test string",
            b"sixth test string",
            b"seventh test string",
        ];

        let dict_data = train_dictionary(&samples, 4096, 15).unwrap();
        let dict = ZstdCompressionDict::new(&dict_data, 15).unwrap();

        // Test data that's similar to dictionary content
        let test_data = b"this is a test string of the fifth order";
        let mut compressed = vec![0u8; max_alloc_for_compress_size(test_data.len())];
        let mut used_copy = false;

        // Compress with dictionary
        let compressed_size =
            compress_with_dictionary(&dict, test_data, &mut compressed, &mut used_copy).unwrap();

        assert!(!used_copy, "Should not fall back to copy compression");
        assert!(compressed_size > 0, "Should produce compressed output");
        assert!(
            compressed_size < test_data.len(),
            "Should achieve some compression"
        );
    }
}
