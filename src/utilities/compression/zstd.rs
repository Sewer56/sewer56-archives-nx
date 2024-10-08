use super::{CompressionResult, DecompressionResult, NxCompressionError, NxDecompressionError};
use crate::api::enums::*;
use core::ffi::c_void;
use zstd_sys::ZSTD_ErrorCode::*;
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

    let result = unsafe {
        ZSTD_compress(
            destination.as_mut_ptr() as *mut c_void,
            destination.len(),
            source.as_ptr() as *const c_void,
            source.len(),
            level,
        )
    };

    let errcode = unsafe { ZSTD_getErrorCode(result) };
    if result > source.len() || errcode == ZSTD_error_dstSize_tooSmall {
        return super::compress(
            CompressionPreference::Copy,
            level,
            source,
            destination,
            used_copy,
        );
    }

    if unsafe { ZSTD_isError(result) } == 0 {
        return Ok(result);
    }

    #[cfg(feature = "zstd_panic_on_unhandled_error")]
    {
        let error_name = ZSTD_getErrorName(error_code);
        panic!(
            "ZStd Compression error: {}",
            CStr::from_ptr(error_name).to_string_lossy()
        );
    }

    #[cfg(not(feature = "zstd_panic_on_unhandled_error"))]
    Err(NxCompressionError::ZStandard(errcode))
}

/// Compresses data with ZStandard, without fallback to Copy
/// if the data is not compressible.
///
/// # Parameters
///
/// * `level`: Level at which we are compressing.
/// * `source`: Length of the source in bytes.
/// * `destination`: Pointer to destination.
pub fn compress_no_copy_fallback(
    level: i32,
    source: &[u8],
    destination: &mut [u8],
) -> CompressionResult {
    let result = unsafe {
        ZSTD_compress(
            destination.as_mut_ptr() as *mut c_void,
            destination.len(),
            source.as_ptr() as *const c_void,
            source.len(),
            level,
        )
    };

    let errcode = unsafe { ZSTD_getErrorCode(result) };
    if unsafe { ZSTD_isError(result) } == 0 {
        return Ok(result);
    }

    #[cfg(feature = "zstd_panic_on_unhandled_error")]
    {
        let error_name = ZSTD_getErrorName(error_code);
        panic!(
            "ZStd Compression error: {}",
            CStr::from_ptr(error_name).to_string_lossy()
        );
    }

    #[cfg(not(feature = "zstd_panic_on_unhandled_error"))]
    Err(NxCompressionError::ZStandard(errcode))
}

/// Decompresses data with ZStandard
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
pub fn decompress(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    unsafe {
        let result = ZSTD_decompress(
            destination.as_mut_ptr() as *mut c_void,
            destination.len(),
            source.as_ptr() as *const c_void,
            source.len(),
        );

        if ZSTD_isError(result) != 0 {
            let error_code = ZSTD_getErrorCode(result);
            Err(NxDecompressionError::ZStandard(error_code))
        } else {
            Ok(result)
        }
    }
}

/// Partially decompresses data with ZStandard until the destination buffer is filled
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
pub fn decompress_partial(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    unsafe {
        let d_stream = ZSTD_createDStream();
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

                #[cfg(feature = "zstd_panic_on_unhandled_error")]
                {
                    let error_name = ZSTD_getErrorName(result);
                    panic!(
                        "ZStd Decompression error: {}",
                        CStr::from_ptr(error_name).to_string_lossy()
                    );
                }

                #[cfg(not(feature = "zstd_panic_on_unhandled_error"))]
                return Err(NxDecompressionError::ZStandard(error_code));
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

/// Represents an error returned from the Nx compression APIs.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GetDecompressedSizeError {
    /// The size of the compressed payload cannot be determined.
    SizeCannotBeDetermined,

    /// Unknown ZStandard error has occurred.
    UnknownErrorOccurred,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn decompress_invalid_data_returns_error() {
        let invalid_compressed_data = vec![0u8; 100];
        let mut destination = vec![0u8; 1000];

        let result = decompress(&invalid_compressed_data, &mut destination);

        assert!(
            result.is_err(),
            "Should return an error for invalid compressed data"
        );
        match result {
            Err(NxDecompressionError::ZStandard(error_code)) => {
                assert_eq!(
                    error_code, ZSTD_error_prefix_unknown,
                    "Not a zstandard file"
                );
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn decompress_partial_invalid_data_returns_error() {
        let invalid_compressed_data = vec![0u8; 100];
        let mut destination = vec![0u8; 1000];

        let result = decompress_partial(&invalid_compressed_data, &mut destination);

        assert!(
            result.is_err(),
            "Should return an error for invalid compressed data"
        );
        match result {
            Err(NxDecompressionError::ZStandard(error_code)) => {
                assert_eq!(
                    error_code, ZSTD_error_prefix_unknown,
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
        let mut used_copy = false;
        let compressed_size =
            compress(3, &original_data, &mut compressed_data, &mut used_copy).unwrap();
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
}
