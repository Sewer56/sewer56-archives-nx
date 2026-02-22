use crate::utilities::compression::copy;

use super::super::{CompressionResult, R3ACompressionError};
use core::ffi::c_void;
use core::ptr::NonNull;
use zstd_sys::ZSTD_ErrorCode::*;
use zstd_sys::*;

/// A wrapper around a ZStandard compression dictionary.
/// This struct is thread-safe and can be shared between threads.
#[derive(Debug)]
pub struct ZstdCompressionDict {
    dict_ptr: *mut ZSTD_CDict,
}

unsafe impl Send for ZstdCompressionDict {}
unsafe impl Sync for ZstdCompressionDict {}

impl ZstdCompressionDict {
    /// Creates a new ZStandard compression dictionary.
    ///
    /// # Parameters
    ///
    /// * `dict_data`: The raw dictionary data.
    /// * `compression_level`: The compression level to optimize the dictionary for.
    pub fn new(dict_data: &[u8], compression_level: i32) -> Result<Self, R3ACompressionError> {
        unsafe {
            let dict_ptr = ZSTD_createCDict(
                dict_data.as_ptr() as *const c_void,
                dict_data.len(),
                compression_level,
            );

            if dict_ptr.is_null() {
                return Err(R3ACompressionError::ZStandard(ZSTD_error_memory_allocation));
            }

            Ok(Self { dict_ptr })
        }
    }

    /// Compresses data using this dictionary.
    ///
    /// # Parameters
    ///
    /// * `source`: The data to compress.
    /// * `level`: The compression level to use.
    /// * `destination`: Buffer to store compressed data.
    /// * `used_copy`: Set to true if copy compression was used due to uncompressible data.
    /// * `cctx`: The ZStandard compression context to use.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `cctx` is a valid ZStandard compression context.
    pub unsafe fn compress(
        &self,
        source: &[u8],
        destination: &mut [u8],
        used_copy: &mut bool,
        cctx: NonNull<ZSTD_CCtx>,
    ) -> CompressionResult {
        *used_copy = false;

        let result = unsafe {
            ZSTD_compress_usingCDict(
                cctx.as_ptr(),
                destination.as_mut_ptr() as *mut c_void,
                destination.len(),
                source.as_ptr() as *const c_void,
                source.len(),
                self.dict_ptr,
            )
        };

        let errcode = unsafe { ZSTD_getErrorCode(result) };
        if result > source.len() || errcode == ZSTD_error_dstSize_tooSmall {
            return copy::compress(source, destination, used_copy);
        }

        if unsafe { ZSTD_isError(result) } == 0 {
            return Ok(result);
        }

        Err(R3ACompressionError::ZStandard(errcode))
    }
}

impl Drop for ZstdCompressionDict {
    fn drop(&mut self) {
        unsafe {
            ZSTD_freeCDict(self.dict_ptr);
        }
    }
}
