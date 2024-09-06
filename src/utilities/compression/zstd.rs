use super::{CompressionResult, DecompressionResult, NxCompressionError, NxDecompressionError};
use crate::api::enums::compression_preference::CompressionPreference;
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

/// Decompresses data with ZStandard
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
pub fn decompress(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    let source_length = source.len();
    let destination_length = destination.len();

    unsafe {
        let d_stream = ZSTD_createDStream();
        let mut out_buf = ZSTD_outBuffer {
            dst: destination.as_mut_ptr() as *mut c_void,
            pos: 0,
            size: destination_length,
        };
        let mut in_buf = ZSTD_inBuffer {
            src: source.as_ptr() as *const c_void,
            pos: 0,
            size: source_length,
        };

        while out_buf.pos < destination_length {
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
