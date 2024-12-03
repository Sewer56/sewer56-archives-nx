use crate::api::traits::*;
use lightweight_mmap::{handles::ReadWriteFileHandle, mmap::ReadWriteMmap};

/// Output data provider that writes data to an existing or new file.
pub struct OutputFileProvider {
    /// The entry from the archive.
    entry: SmallFileEntry,
    /// The entry from the archive.
    file_handle: ReadWriteFileHandle,
}

impl OutputFileProvider {
    /// Creates a new provider for the given file path
    pub fn new(path: &str, entry: SmallFileEntry) -> Result<Self, FileOutputError> {
        let file_handle =
            ReadWriteFileHandle::create_preallocated(path, entry.decompressed_size as i64)?;
        Ok(Self { entry, file_handle })
    }
}

impl OutputDataProvider for OutputFileProvider {
    fn entry(&self) -> SmallFileEntry {
        self.entry
    }

    fn get_file_data<'a>(
        &'a self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn ReadWriteFileData + 'a>, FileProviderError> {
        let mapping = ReadWriteMmap::new(&self.file_handle, start, length as usize)?;
        Ok(Box::new(ReadWriteMappedFileData::new(mapping)))
    }
}

/// A struct that implements FileData trait for memory mapped regions of a file
pub struct ReadWriteMappedFileData<'a> {
    mapping: ReadWriteMmap<'a>,
}

impl<'a> ReadWriteMappedFileData<'a> {
    /// Creates a new mapped file data instance
    pub fn new(mapping: ReadWriteMmap<'a>) -> Self {
        Self { mapping }
    }
}

impl ReadWriteFileData for ReadWriteMappedFileData<'_> {
    fn data(&mut self) -> &mut [u8] {
        self.mapping.as_mut_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::*;
    use tempfile::tempdir;

    #[test]
    #[cfg_attr(miri, ignore)] // involves external I/O
    fn new_provider() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.bin").to_str().unwrap().to_string();
        let entry = SmallFileEntry::new(100, 0, 0);

        let provider = OutputFileProvider::new(&file_path, entry).unwrap();
        assert_eq!(provider.entry(), entry);

        // Verify file was created with correct size
        let metadata = metadata(&file_path).unwrap();
        assert_eq!(metadata.len(), 100);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn new_provider_zero_size() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("empty.bin").to_str().unwrap().to_string();
        let entry = SmallFileEntry::new(0, 0, 0);

        let provider = OutputFileProvider::new(&file_path, entry).unwrap();
        assert_eq!(provider.entry(), entry);

        // Verify empty file was created
        let metadata = metadata(&file_path).unwrap();
        assert_eq!(metadata.len(), 0);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn get_full_range() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("full.bin").to_str().unwrap().to_string();
        let entry = SmallFileEntry::new(100, 0, 0);

        let provider = OutputFileProvider::new(&file_path, entry).unwrap();
        let mut file_data = provider.get_file_data(0, 100).unwrap();
        assert_eq!(file_data.data().len(), 100);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn get_partial_range() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("partial.bin").to_str().unwrap().to_string();
        let entry = SmallFileEntry::new(100, 0, 0);

        let provider = OutputFileProvider::new(&file_path, entry).unwrap();
        let mut file_data = provider.get_file_data(50, 30).unwrap();
        assert_eq!(file_data.data().len(), 30);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn write_and_read_data() {
        let dir = tempdir().unwrap();
        let file_path = dir
            .path()
            .join("write_read.bin")
            .to_str()
            .unwrap()
            .to_string();
        let entry = SmallFileEntry::new(10, 0, 0);
        let provider = OutputFileProvider::new(&file_path, entry).unwrap();

        let test_data = [1, 2, 3, 4, 5];
        let mut file_data = provider.get_file_data(0, 5).unwrap();
        file_data.data().copy_from_slice(&test_data);

        let mut file_data = provider.get_file_data(0, 5).unwrap();
        assert_eq!(file_data.data(), &test_data);

        // Verify data was written to disk
        let file_contents = read(&file_path).unwrap();
        assert_eq!(&file_contents[0..5], &test_data);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn multiple_range_writes() {
        let dir = tempdir().unwrap();
        let file_path = dir
            .path()
            .join("multi_write.bin")
            .to_str()
            .unwrap()
            .to_string();
        let entry = SmallFileEntry::new(10, 0, 0);
        let provider = OutputFileProvider::new(&file_path, entry).unwrap();

        let mut file_data = provider.get_file_data(0, 5).unwrap();
        file_data.data().copy_from_slice(&[1, 2, 3, 4, 5]);

        let mut file_data = provider.get_file_data(5, 5).unwrap();
        file_data.data().copy_from_slice(&[6, 7, 8, 9, 10]);

        let mut file_data = provider.get_file_data(0, 10).unwrap();
        assert_eq!(file_data.data(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        // Verify complete file contents
        let file_contents = read(&file_path).unwrap();
        assert_eq!(file_contents, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn invalid_file_path() {
        let entry = SmallFileEntry::new(10, 0, 0);
        let result = OutputFileProvider::new("/nonexistent/directory/file.bin", entry);
        assert!(matches!(
            result,
            Err(FileOutputError::FileHandleOpenError(_))
        ));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn verify_send() {
        fn assert_send<T: Send>() {}
        assert_send::<OutputFileProvider>();
    }
}
