mod compress;
mod decompress;
mod train;

pub use compress::ZstdCompressionDict;
pub use decompress::ZstdDecompressionDict;
pub use train::*;

#[cfg(test)]
mod tests {
    use core::ptr::NonNull;

    use super::*;
    use alloc::vec;
    use zstd_sys::*;
    const COMPRESSION_LEVEL: i32 = 1;

    #[test]
    fn can_train_and_use_dictionary() {
        unsafe {
            // Create sample data for dictionary training
            let samples: [&[u8]; 7] = [
                b"This is sample text for training",
                b"More sample text for the dictionary",
                b"Yet another sample for training",
                b"You can do cool stuff using Reloaded",
                b"Training data is very cool",
                b"This is a catastrophe",
                b"Or is it?",
            ];

            // Train dictionary
            let dict_data = train_dictionary(&samples, 4096, COMPRESSION_LEVEL).unwrap();

            // Create compression and decompression dictionaries
            let comp_dict = ZstdCompressionDict::new(&dict_data, 3).unwrap();
            let decomp_dict = ZstdDecompressionDict::new(&dict_data).unwrap();

            // Create contexts
            let cctx: NonNull<ZSTD_CCtx_s> = NonNull::new(ZSTD_createCCtx()).unwrap();
            let dctx = NonNull::new(ZSTD_createDCtx()).unwrap();

            // Test compression/decompression
            let test_data =
                b"This is a test using the trained dictionary. Training data is pretty cool.";
            let mut compressed =
                vec![0u8; super::super::max_alloc_for_compress_size(test_data.len())];
            let mut decompressed = vec![0u8; test_data.len()];
            let mut used_copy = false;

            // Compress
            let compressed_size = comp_dict
                .compress(test_data, &mut compressed, &mut used_copy, cctx)
                .unwrap();
            compressed.truncate(compressed_size);

            // Decompress
            let decompressed_size = decomp_dict
                .decompress(&compressed, &mut decompressed, dctx)
                .unwrap();
            decompressed.truncate(decompressed_size);

            // Verify decompression
            assert_eq!(test_data, &decompressed[..]);

            // Clean up contexts
            ZSTD_freeCCtx(cctx.as_ptr());
            ZSTD_freeDCtx(dctx.as_ptr());
        }
    }

    #[test]
    fn handles_incompressible_data() {
        unsafe {
            let dict_data = vec![0u8; 1024]; // Empty dictionary
            let dict = ZstdCompressionDict::new(&dict_data, 3).unwrap();

            let cctx = NonNull::new(ZSTD_createCCtx()).unwrap();

            let incompressible_data = b"randomdata123"; // Small, random-like data
            let mut compressed =
                vec![0u8; super::super::max_alloc_for_compress_size(incompressible_data.len())];
            let mut used_copy = false;

            let compressed_size = dict
                .compress(incompressible_data, &mut compressed, &mut used_copy, cctx)
                .unwrap();

            assert!(
                used_copy,
                "Should fall back to copy for incompressible data"
            );
            assert_eq!(compressed_size, incompressible_data.len());

            ZSTD_freeCCtx(cctx.as_ptr());
        }
    }
}
