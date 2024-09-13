use super::{DecompressionResult, NxDecompressionError};
use core::ffi::c_void;
use zstd_sys::ZSTD_ErrorCode::ZSTD_error_memory_allocation;
use zstd_sys::*;

/// A streaming decompressor for ZStandard-compressed data.
///
/// This struct allows for chunk-by-chunk decompression of ZStandard-compressed data,
/// which is useful for processing large compressed streams or in memory-constrained environments.
pub struct ZstdDecompressor<'a> {
    d_stream: *mut ZSTD_DStream,
    in_buf: ZSTD_inBuffer,

    // Ensure 'source' is not dropped before 'd_stream'
    _source: &'a [u8],
}

impl<'a> ZstdDecompressor<'a> {
    /// Creates a new `ZstdDecompressor` instance.
    ///
    /// # Arguments
    ///
    /// * `source` - A byte slice containing the ZStandard-compressed data to decompress.
    ///
    /// # Returns
    ///
    /// Returns a `Result` which is:
    /// * `Ok(ZstdDecompressor)` on success
    /// * `Err(NxDecompressionError)` if the decompressor couldn't be created (usually due to memory allocation failure)
    pub fn new(source: &'a [u8]) -> Result<Self, NxDecompressionError> {
        unsafe {
            let d_stream = ZSTD_createDStream();
            if d_stream.is_null() {
                return Err(NxDecompressionError::ZStandard(
                    ZSTD_error_memory_allocation,
                ));
            }

            Ok(ZstdDecompressor {
                d_stream,
                in_buf: ZSTD_inBuffer {
                    src: source.as_ptr() as *const c_void,
                    size: source.len(),
                    pos: 0,
                },
                _source: source,
            })
        }
    }

    /// Decompresses the next chunk of data.
    ///
    /// This method decompresses as much data as possible into the provided destination buffer.
    /// It may need to be called multiple times to fully decompress all the data.
    ///
    /// # Arguments
    ///
    /// * `destination` - A mutable byte slice where the decompressed data will be written.
    ///
    /// # Returns
    ///
    /// Returns a `DecompressionResult` which is:
    /// * `Ok(usize)` - The number of bytes written to the destination buffer
    /// * `Err(NxDecompressionError)` if an error occurred during decompression
    pub fn decompress_chunk(&mut self, destination: &mut [u8]) -> DecompressionResult {
        let mut out_buf = ZSTD_outBuffer {
            dst: destination.as_mut_ptr() as *mut c_void,
            size: destination.len(),
            pos: 0,
        };

        unsafe {
            let result = ZSTD_decompressStream(self.d_stream, &mut out_buf, &mut self.in_buf);

            if ZSTD_isError(result) != 0 {
                return Err(NxDecompressionError::ZStandard(ZSTD_getErrorCode(result)));
            }
        }

        Ok(out_buf.pos)
    }

    /// Checks if all input data has been consumed.
    ///
    /// # Returns
    ///
    /// Returns `true` if all input data has been processed, `false` otherwise.
    pub fn is_finished(&self) -> bool {
        self.in_buf.pos == self.in_buf.size
    }

    /// Resets the decompressor to start processing from the beginning of the input.
    ///
    /// This method allows you to restart decompression from the beginning of the original input
    /// without creating a new `ZstdDecompressor` instance.
    pub fn reset(&mut self) {
        self.in_buf.pos = 0;
    }
}

impl Drop for ZstdDecompressor<'_> {
    fn drop(&mut self) {
        unsafe {
            ZSTD_freeDStream(self.d_stream);
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::utilities::compression::zstd::*;
    use alloc::*;
    use vec::Vec;

    fn create_compressed_data(data: &[u8], level: i32) -> Vec<u8> {
        let mut compressed = vec![0u8; max_alloc_for_compress_size(data.len())];
        let mut used_copy = false;
        let compressed_size = compress(level, data, &mut compressed, &mut used_copy).unwrap();
        compressed.truncate(compressed_size);
        compressed
    }

    #[test]
    fn can_decompress_basic_data() {
        let original_data = b"Hello, ZStandard!".repeat(100);
        let compressed_data = create_compressed_data(&original_data, 3);

        let mut decompressor = ZstdDecompressor::new(&compressed_data).unwrap();
        let mut decompressed = vec![0u8; original_data.len()];

        let size = decompressor.decompress_chunk(&mut decompressed).unwrap();

        assert_eq!(size, original_data.len());
        assert_eq!(&decompressed[..size], &original_data[..]);
        assert!(decompressor.is_finished());
    }

    #[test]
    fn can_decompress_in_chunks() {
        let original_data = b"Chunked decompression test".repeat(1000);
        let compressed_data = create_compressed_data(&original_data, 3);

        let mut decompressor = ZstdDecompressor::new(&compressed_data).unwrap();
        let mut decompressed = Vec::new();
        let mut buffer = vec![0u8; 100];

        while !decompressor.is_finished() {
            let size = decompressor.decompress_chunk(&mut buffer).unwrap();
            decompressed.extend_from_slice(&buffer[..size]);
        }

        assert_eq!(decompressed, original_data);
    }

    #[test]
    fn can_reset_and_reuse_decompressor() {
        let original_data = b"Reset test data".repeat(50);
        let compressed_data = create_compressed_data(&original_data, 3);

        let mut decompressor = ZstdDecompressor::new(&compressed_data).unwrap();
        let mut decompressed1 = vec![0u8; original_data.len()];
        let mut decompressed2 = vec![0u8; original_data.len()];

        let size1 = decompressor.decompress_chunk(&mut decompressed1).unwrap();
        assert_eq!(&decompressed1[..size1], &original_data[..]);

        decompressor.reset();

        let size2 = decompressor.decompress_chunk(&mut decompressed2).unwrap();
        assert_eq!(&decompressed2[..size2], &original_data[..]);
        assert_eq!(size1, size2);
    }

    #[test]
    fn handles_invalid_data() {
        let invalid_data = vec![0u8; 100];
        let mut decompressor = ZstdDecompressor::new(&invalid_data).unwrap();
        let mut output = vec![0u8; 1000];

        let result = decompressor.decompress_chunk(&mut output);
        assert!(result.is_err());
    }

    #[test]
    fn works_with_small_output_buffer() {
        let original_data = b"Small buffer test".repeat(100);
        let compressed_data = create_compressed_data(&original_data, 3);

        let mut decompressor = ZstdDecompressor::new(&compressed_data).unwrap();
        let mut small_buffer = vec![0u8; 10];

        let size = decompressor.decompress_chunk(&mut small_buffer).unwrap();
        assert_eq!(size, 10);
        assert_eq!(&small_buffer[..size], &original_data[..size]);
        assert!(!decompressor.is_finished());
    }

    #[test]
    fn can_decompress_empty_input() {
        let empty_data = create_compressed_data(&[], 3);
        let mut decompressor = ZstdDecompressor::new(&empty_data).unwrap();
        let mut output = vec![0u8; 10];

        let size = decompressor.decompress_chunk(&mut output).unwrap();
        assert_eq!(size, 0);
        assert!(decompressor.is_finished());
    }
}
