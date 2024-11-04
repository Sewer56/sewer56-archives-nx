use super::LazyDecompressedSolidNxBlock;
use crate::{api::traits::*, headers::managed::FileEntry};
use std::sync::Arc;

/// This provider allows you to read a file from an existing SOLID Nx block,
/// where the Nx block is lazy decompressed and provided by [`LazyDecompressedSolidNxBlock`].
///
/// This is used when you only want some files from an exising block in a
/// foreign Nx archive.
///
/// # Remarks
///
/// This struct is for internal use only, some assumptions are made, such that [`Self::get_file_data`]
/// is only called with start 0 and length equal to file size. The library asserts this in debug mode
pub struct FromExistingNxBlock {
    block: Arc<LazyDecompressedSolidNxBlock>,
    block_offset: u32,
    file_size: u32,
}

impl FromExistingNxBlock {
    pub fn new(block: Arc<LazyDecompressedSolidNxBlock>, entry: &FileEntry) -> Self {
        Self {
            block,
            block_offset: entry.decompressed_block_offset,
            // Note: Size of the file is constrained by MAX_BLOCK_SIZE
            // so we can represent this as u32. Making MAX_BLOCK_SIZE larger is a compile time failure
            file_size: entry.decompressed_size as u32,
        }
    }
}

impl InputDataProvider for FromExistingNxBlock {
    fn get_file_data<'a>(
        &'a self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn ReadOnlyFileData + 'a>, FileProviderError> {
        debug_assert!(start == 0, "start must be 0");
        debug_assert!(
            length == self.file_size as u64,
            "file size must match file size"
        );

        let base_data_ptr = self.block.get_data()?;
        let base_data_ptr = unsafe {
            base_data_ptr.get_unchecked(
                self.block_offset as usize..self.block_offset as usize + self.file_size as usize,
            )
        };
        Ok(Box::new(DecompressedNxBlockFileData::new(base_data_ptr)))
    }
}

pub struct DecompressedNxBlockFileData<'a> {
    /// Raw pointer to the data
    data: &'a [u8],
}

impl<'a> DecompressedNxBlockFileData<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl ReadOnlyFileData for DecompressedNxBlockFileData<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{enums::CompressionPreference, filedata::FromStreamProvider};
    use std::io::Cursor;

    /// Helper function to create a test block with given data
    fn create_test_block(data: Vec<u8>) -> Arc<LazyDecompressedSolidNxBlock> {
        let len = data.len();
        let provider = Arc::new(FromStreamProvider::new(Cursor::new(data)));
        let block =
            LazyDecompressedSolidNxBlock::new(provider, 0, len as u32, CompressionPreference::Copy);

        Arc::new(block)
    }

    #[test]
    fn can_create_provider() {
        let block = create_test_block(vec![1, 2, 3, 4, 5]);
        let entry = FileEntry {
            decompressed_block_offset: 0,
            decompressed_size: 5,
            ..Default::default()
        };

        let provider = FromExistingNxBlock::new(block, &entry);
        assert_eq!(provider.block_offset, 0);
        assert_eq!(provider.file_size, 5);
    }

    #[test]
    fn can_read_file_data() {
        let block = create_test_block(vec![1, 2, 3, 4, 5]);
        let entry = FileEntry {
            decompressed_block_offset: 0,
            decompressed_size: 5,
            ..Default::default()
        };

        block.consider_file(&entry);
        let provider = FromExistingNxBlock::new(block, &entry);
        let file_data = provider.get_file_data(0, 5).unwrap();
        assert_eq!(file_data.data(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn can_read_with_offset() {
        let block = create_test_block(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let entry = FileEntry {
            decompressed_block_offset: 2, // Start at third byte
            decompressed_size: 4,         // Take 4 bytes
            ..Default::default()
        };

        block.consider_file(&entry);
        let provider = FromExistingNxBlock::new(block, &entry);
        let file_data = provider.get_file_data(0, 4).unwrap();
        assert_eq!(file_data.data(), &[3, 4, 5, 6]);
    }

    #[test]
    fn multiple_providers_share_block() {
        let block = create_test_block(vec![1, 2, 3, 4, 5, 6, 7, 8]);

        let entry1 = FileEntry {
            decompressed_block_offset: 0,
            decompressed_size: 3,
            ..Default::default()
        };

        let entry2 = FileEntry {
            decompressed_block_offset: 3,
            decompressed_size: 3,
            ..Default::default()
        };

        block.consider_file(&entry1);
        block.consider_file(&entry2);

        let provider1 = FromExistingNxBlock::new(Arc::clone(&block), &entry1);
        let provider2 = FromExistingNxBlock::new(block, &entry2);

        let data1 = provider1.get_file_data(0, 3).unwrap();
        let data2 = provider2.get_file_data(0, 3).unwrap();

        assert_eq!(data1.data(), &[1, 2, 3]);
        assert_eq!(data2.data(), &[4, 5, 6]);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "start must be 0")]
    fn panics_on_non_zero_start() {
        let block = create_test_block(vec![1, 2, 3, 4, 5]);
        let entry = FileEntry {
            decompressed_block_offset: 0,
            decompressed_size: 5,
            ..Default::default()
        };

        let provider = FromExistingNxBlock::new(block, &entry);
        let _ = provider.get_file_data(1, 4); // Should panic - start must be 0
    }

    #[test]
    fn handles_empty_file() {
        let block = create_test_block(vec![1, 2, 3]);
        let entry = FileEntry {
            decompressed_block_offset: 0,
            decompressed_size: 0,
            ..Default::default()
        };

        block.consider_file(&entry);
        let provider = FromExistingNxBlock::new(block, &entry);
        let file_data = provider.get_file_data(0, 0).unwrap();
        assert_eq!(file_data.data().len(), 0);
    }
}
