use crate::api::traits::*;
use lightweight_mmap::handles::ReadOnlyFileHandle;
use lightweight_mmap::mmap::ReadOnlyMmap;
use nanokit::string_concat_unsafe::*;
use std::path::*;

/// A struct that implements file data provider functionality for files on disk.
/// Each provider instance corresponds to a single file and is accessed by only one thread at a time.
pub struct FromFilePathProvider {
    file_handle: ReadOnlyFileHandle,
}

impl FromFilePathProvider {
    /// Creates a new provider for the given file path
    pub fn new(path: &str) -> Result<Self, FileProviderError> {
        let file_handle = ReadOnlyFileHandle::open(path)?;
        Ok(Self { file_handle })
    }

    /// Creates a new provider by combining a directory path and file name
    pub fn new_from_dir(dir: &str, file: &str) -> Result<Self, FileProviderError> {
        // Combine paths with a platform-specific separator while avoiding allocation
        let path = if dir.ends_with(MAIN_SEPARATOR) {
            unsafe { concat_2_no_overflow(dir, file) }
        } else {
            unsafe { concat_3_no_overflow(dir, MAIN_SEPARATOR_STR, file) }
        };

        Self::new(&path)
    }

    /// Returns the size of the file behind the provider
    pub fn file_size(&self) -> Result<i64, FileProviderError> {
        self.file_handle.size().map_err(|op| op.into())
    }
}

impl InputDataProvider for FromFilePathProvider {
    fn get_file_data<'a>(
        &'a self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn ReadOnlyFileData + 'a>, FileProviderError> {
        let mapping = ReadOnlyMmap::new(&self.file_handle, start, length as usize)?;
        Ok(Box::new(ReadOnlyMappedFileData::new(mapping)))
    }
}

/// A struct that implements FileData trait for memory mapped regions of a file
pub struct ReadOnlyMappedFileData<'a> {
    mapping: ReadOnlyMmap<'a>,
}

impl<'a> ReadOnlyMappedFileData<'a> {
    /// Creates a new mapped file data instance
    pub fn new(mapping: ReadOnlyMmap<'a>) -> Self {
        Self { mapping }
    }
}

impl<'a> ReadOnlyFileData for ReadOnlyMappedFileData<'a> {
    fn data(&self) -> &'a [u8] {
        self.mapping.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    #[cfg_attr(miri, ignore)] // involves external I/O
    fn can_create_provider() {
        let temp_file = NamedTempFile::new().unwrap();
        let _ = FromFilePathProvider::new(temp_file.path().to_str().unwrap()).unwrap();
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn can_create_provider_from_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        write(&file_path, b"Test data").unwrap();

        let provider =
            FromFilePathProvider::new_from_dir(temp_dir.path().to_str().unwrap(), "test.txt")
                .unwrap();
        let data = provider.get_file_data(0, 9).unwrap();
        assert_eq!(data.data(), b"Test data");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn errors_on_nonexistent_file() {
        let result = FromFilePathProvider::new("nonexistent_file.txt");
        assert!(matches!(
            result.err(),
            Some(FileProviderError::FileHandleOpenError { .. })
        ));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn can_read_file_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let provider = FromFilePathProvider::new(temp_file.path().to_str().unwrap()).unwrap();
        let data = provider.get_file_data(0, 13).unwrap();
        assert_eq!(data.data(), b"Hello, World!");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn can_read_file_data_with_offset() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let provider = FromFilePathProvider::new(temp_file.path().to_str().unwrap()).unwrap();
        let data = provider.get_file_data(7, 5).unwrap();
        assert_eq!(data.data(), b"World");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn can_create_multiple_mappings() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let provider = FromFilePathProvider::new(temp_file.path().to_str().unwrap()).unwrap();

        let data1 = provider.get_file_data(0, 5).unwrap();
        let data2 = provider.get_file_data(7, 5).unwrap();

        assert_eq!(data1.data(), b"Hello");
        assert_eq!(data2.data(), b"World");
    }
}
