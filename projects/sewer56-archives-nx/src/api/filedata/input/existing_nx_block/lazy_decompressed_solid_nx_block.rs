use crate::api::packing::packing_settings::MAX_BLOCK_SIZE;
use crate::api::{enums::CompressionPreference, traits::*};
use crate::headers::managed::FileEntry;
use crate::prelude::*;
use crate::utilities::compression;
use alloc::sync::Arc;
use core::cell::UnsafeCell;
use once_cell::sync::OnceCell;

/// Represents a block of data from an existing Nx archive that is lazily decompressed
/// on demand.
///
/// This is used when you only want some files from an exising block in a
/// foreign Nx archive. Namely, you have a SOLID block, but only need to lift out some
/// files from it.
///
/// # Usage
///
/// Usage is as follows.
///
/// 1. Create a new [`LazyDecompressedSolidNxBlock`] instance
/// 2. Call [`LazyDecompressedSolidNxBlock::consider_file`] for each file you want to extract.
/// 3. Call [`LazyDecompressedSolidNxBlock::get_data`] to get the decompressed data.
///     - This will decompress if not already decompressed, or reuse previous decompressed state.
///
/// # Remarks
///
/// This struct is for SOLID blocks only, whose block size is bound by [`MAX_BLOCK_SIZE`].
/// This maxes maximum memory size bound by [`MAX_BLOCK_SIZE`], so we can represent this as [`u32`].
pub struct LazyDecompressedSolidNxBlock {
    /// Raw decompressed data, lazily initialized when needed
    data: OnceCell<Box<[u8]>>,

    /// Provides access to the original Nx archive
    source_nx_data_provider: Arc<dyn InputDataProvider + Send + Sync>,

    /// Offset of the block in original NX archive (via `source_nx_data_provider`)
    block_offset: u64,

    /// Length of the block in the original NX archive (via `source_nx_data_provider`)
    compressed_block_length: u32,

    /// Number of bytes that need decompressing for all files that will be extracted
    /// from this block
    num_bytes_to_decompress: UnsafeCell<u32>,

    /// Compression used with the original NX archive (via `source_nx_data_provider`)
    compression: CompressionPreference,
}

// These are Send and Sync because all operations via methods are Send and Sync via [`OnceLock`].
unsafe impl Send for LazyDecompressedSolidNxBlock {}
unsafe impl Sync for LazyDecompressedSolidNxBlock {}

impl LazyDecompressedSolidNxBlock {
    /// Creates a new [`LazyDecompressedSolidNxBlock`]
    ///
    /// # Arguments
    /// * `source_nx_data_provider` - Provides raw access to the original NX file
    /// * `block_offset` - Byte offset of block in source provider  
    /// * `compressed_block_length` - Length of block in source provider
    /// * `compression` - Compression used by source block
    pub fn new(
        source_nx_data_provider: Arc<dyn InputDataProvider + Send + Sync>,
        block_offset: u64,
        compressed_block_length: u32,
        compression: CompressionPreference,
    ) -> Self {
        Self {
            data: OnceCell::new(),
            source_nx_data_provider,
            num_bytes_to_decompress: UnsafeCell::new(0),
            block_offset,
            compressed_block_length,
            compression,
        }
    }

    /// Creates a new [`LazyDecompressedSolidNxBlock`]
    ///
    /// # Arguments
    /// * `source_nx_data_provider` - Provides raw access to the original NX file
    /// * `block_offset` - Byte offset of block in source provider  
    /// * `compressed_block_length` - Length of block in source provider
    /// * `compression` - Compression used by source block
    pub fn new_arc(
        source_nx_data_provider: Arc<dyn InputDataProvider + Send + Sync>,
        block_offset: u64,
        compressed_block_length: u32,
        compression: CompressionPreference,
    ) -> Arc<Self> {
        Arc::new(Self::new(
            source_nx_data_provider,
            block_offset,
            compressed_block_length,
            compression,
        ))
    }

    /// Updates number of bytes needed for decompression based on a file entry
    /// from the archive.
    pub fn consider_file(&self, entry: &FileEntry) {
        let max_offset = entry.decompressed_block_offset + entry.decompressed_size as u32;
        debug_assert!(max_offset <= MAX_BLOCK_SIZE);
        unsafe {
            if max_offset > *self.num_bytes_to_decompress.get() {
                *self.num_bytes_to_decompress.get() = max_offset
            }
        }
    }

    /// Gets raw data pointer, decompressing if needed
    ///
    /// # Returns
    /// NonNull pointer to decompressed data
    pub fn get_data(&self) -> Result<&[u8], FileProviderError> {
        self.data
            .get_or_try_init(|| {
                // Do decompression
                let num_bytes = unsafe { (*self.num_bytes_to_decompress.get()) as usize };
                let mut decompressed = unsafe { Box::new_uninit_slice(num_bytes).assume_init() };

                // Get compressed data
                let compressed = self
                    .source_nx_data_provider
                    .get_file_data(self.block_offset, self.compressed_block_length as u64)?;

                // Decompress
                compression::decompress_partial(
                    self.compression,
                    compressed.data(),
                    &mut decompressed,
                )?;

                Ok(decompressed)
            })
            .map(|x| x as &[u8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::filedata::FromStreamProvider;
    use crate::prelude::vec;
    use std::io::Cursor;

    #[test]
    fn data_sharing_works() {
        let provider = Arc::new(FromStreamProvider::new(Cursor::new(vec![0u8; 100])));
        let block = Arc::new(LazyDecompressedSolidNxBlock::new(
            provider,
            0,
            100,
            CompressionPreference::Copy,
        ));
        let block2 = Arc::clone(&block);

        block.consider_file(&FileEntry {
            decompressed_block_offset: 0,
            decompressed_size: 100,
            ..Default::default()
        });

        let data1 = block.get_data().unwrap();
        let data2 = block2.get_data().unwrap();
        assert_eq!(data1.as_ptr(), data2.as_ptr());
    }
}
