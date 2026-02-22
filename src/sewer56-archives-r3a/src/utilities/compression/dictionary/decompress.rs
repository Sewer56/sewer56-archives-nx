use super::super::{DecompressionResult, R3ADecompressionError};
use core::ffi::c_void;
use core::ptr::NonNull;
use zstd_sys::ZSTD_ErrorCode::*;
use zstd_sys::*;

/// A wrapper around a ZStandard decompression dictionary.
/// This struct is thread-safe and can be shared between threads.
#[derive(Debug)]
pub struct ZstdDecompressionDict {
    dict_ptr: *mut ZSTD_DDict,
}

impl ZstdDecompressionDict {
    /// Creates a new ZStandard decompression dictionary.
    ///
    /// # Parameters
    ///
    /// * `dict_data`: The raw dictionary data.
    pub fn new(dict_data: &[u8]) -> Result<Self, R3ADecompressionError> {
        unsafe {
            let dict_ptr = ZSTD_createDDict(dict_data.as_ptr() as *const c_void, dict_data.len());

            if dict_ptr.is_null() {
                return Err(R3ADecompressionError::ZStandard(
                    ZSTD_error_memory_allocation,
                ));
            }

            Ok(Self { dict_ptr })
        }
    }

    /// Decompresses data using this dictionary.
    ///
    /// # Parameters
    ///
    /// * `source`: The compressed data.
    /// * `destination`: Buffer to store decompressed data.
    /// * `dctx`: The ZStandard decompression context to use.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `dctx` is a valid ZStandard decompression context.
    pub unsafe fn decompress(
        &self,
        source: &[u8],
        destination: &mut [u8],
        dctx: NonNull<ZSTD_DCtx>,
    ) -> DecompressionResult {
        unsafe {
            let result = ZSTD_decompress_usingDDict(
                dctx.as_ptr(),
                destination.as_mut_ptr() as *mut c_void,
                destination.len(),
                source.as_ptr() as *const c_void,
                source.len(),
                self.dict_ptr,
            );

            if ZSTD_isError(result) != 0 {
                let error_code = ZSTD_getErrorCode(result);
                Err(R3ADecompressionError::ZStandard(error_code))
            } else {
                Ok(result)
            }
        }
    }
}

unsafe impl Send for ZstdDecompressionDict {}

impl Drop for ZstdDecompressionDict {
    fn drop(&mut self) {
        unsafe {
            ZSTD_freeDDict(self.dict_ptr);
        }
    }
}
