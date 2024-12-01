use super::{enums::*, filedata::*, packing::packer_file::PackerFile, traits::*};
use crate::{
    api::packing::packing_settings::PackingSettings, utilities::io::file_finder::find_files,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use std::io::{Read, Seek};

/// A builder pattern implementation for creating NX archives.
///
/// This builder provides a fluent interface for:
///
/// - Adding files to be packed into the archive
/// - Configuring compression settings and algorithms
/// - Setting block and chunk sizes for optimal compression
/// - Enabling or disabling file deduplication
/// - Building the final archive
///
/// The builder ensures all parameters are validated and normalized before creating
/// the archive, making it impossible to create an invalid archive through this interface.
///
/// # Example
///
/// ```no_run
/// use sewer56_archives_nx::api::packer_builder::*;
/// let builder = NxPackerBuilder::new()
///     .with_block_size(1048576)
///     .with_chunk_size(4194304)
///     .with_chunked_deduplication(true);
/// ```
pub struct NxPackerBuilder<'a> {
    /// Settings for the packer, including compression preferences,
    /// block sizes, and deduplication options.
    pub settings: PackingSettings,

    /// Collection of files to be included in the archive.
    pub files: Vec<PackerFile<'a>>,

    /// Phantom data to track the lifetime of referenced slices
    _phantom: PhantomData<&'a [u8]>,
}

impl<'a> NxPackerBuilder<'a> {
    /// Creates a new builder instance with default settings.
    ///
    /// The default settings are optimized for general use cases, providing
    /// a balance between compression ratio and performance. You can customize
    /// these settings using the builder methods.
    ///
    /// # Returns
    ///
    /// Returns a new `NxPackerBuilder` instance with default settings initialized.
    pub fn new() -> Self {
        Self {
            settings: PackingSettings::default(),
            files: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Creates a new builder instance with custom externally set settings.
    ///
    /// # Returns
    ///
    /// Returns a new `NxPackerBuilder` instance with externally set settings.
    pub fn with_settings(settings: PackingSettings) -> Self {
        Self {
            settings,
            files: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Adds a file from the local filesystem to be packed.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file on the local filesystem.
    /// * `options` - Parameters controlling how the file should be packed.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be accessed or if there are issues reading its metadata.
    pub fn add_file(
        &mut self,
        file_path: &str,
        options: AddFileParams,
    ) -> Result<&mut Self, FileProviderError> {
        let file = PackerFile::from_file_path_with_unknown_size(file_path, options.relative_path)?;
        self.files.push(file);
        Ok(self)
    }

    /// Adds a file from a byte array to be packed.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes of the file.
    /// * `options` - Parameters controlling how the file should be packed.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    pub fn add_file_from_byte_slice(
        &mut self,
        data: &'a [u8],
        options: AddFileParams,
    ) -> &mut Self {
        let provider = Box::new(FromSliceReferenceProvider::new(data));
        let file = PackerFile::new(options.relative_path, data.len() as u64, provider)
            .with_compression(options.compression_preference)
            .with_solid(options.solid_type);

        self.files.push(file);
        self
    }

    /// Adds a file from a byte array to be packed.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes of the file.
    /// * `options` - Parameters controlling how the file should be packed.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    pub fn add_file_from_boxed_slice(
        &mut self,
        data: Box<[u8]>,
        options: AddFileParams,
    ) -> &mut Self {
        let len = data.len();
        let provider = Box::new(FromBoxedSliceProvider::new(data));
        let file = PackerFile::new(options.relative_path, len as u64, provider)
            .with_compression(options.compression_preference)
            .with_solid(options.solid_type);

        self.files.push(file);
        self
    }

    /// Adds a file from a stream to be packed.
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream containing the file data. Must support seeking.
    /// * `length` - The length of data to read from the stream, starting from the current position.
    /// * `options` - Parameters controlling how the file should be packed.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    pub fn add_file_from_stream<T: Read + Seek + Send + 'static>(
        &mut self,
        stream: T,
        length: u64,
        options: AddFileParams,
    ) -> &mut Self {
        let provider = Box::new(FromStreamProvider::new(stream));
        let file = PackerFile::new(options.relative_path, length, provider)
            .with_compression(options.compression_preference)
            .with_solid(options.solid_type);

        self.files.push(file);
        self
    }

    /// Adds all files under a given directory to the archive.
    ///
    /// Files will be added recursively, maintaining their relative paths.
    /// The paths in the archive will be relative to the provided folder.
    ///
    /// # Arguments
    ///
    /// * `folder` - The directory to add files from.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be accessed or if there are issues reading file metadata.
    pub fn add_folder(&mut self, folder: &str) -> Result<&mut Self, FileProviderError> {
        find_files(folder, |file| self.files.push(file))?;
        Ok(self)
    }

    /// Sets the size of SOLID blocks used in the archive.
    ///
    /// SOLID blocks combine multiple small files into a single compressed unit,
    /// which can significantly improve compression ratios. The block size determines
    /// the maximum size of these combined units.
    ///
    /// # Arguments
    ///
    /// * `block_size` - Size of SOLID blocks in bytes. Must be between
    ///   [`MIN_BLOCK_SIZE`] and [`MAX_BLOCK_SIZE`]. The value will be
    ///   automatically adjusted to the nearest power of 2 minus 1.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    ///
    /// [`MIN_BLOCK_SIZE`]: crate::api::packing::packing_settings::MIN_BLOCK_SIZE
    /// [`MAX_BLOCK_SIZE`]: crate::api::packing::packing_settings::MAX_BLOCK_SIZE
    pub fn with_block_size(mut self, block_size: u32) -> Self {
        self.settings.block_size = block_size;
        self
    }

    /// Sets the size of chunks used when splitting large files.
    ///
    /// Large files are split into chunks to allow for parallel compression
    /// and decompression. This setting controls the size of those chunks.
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - Size of chunks in bytes. Must be between
    ///   [`MIN_CHUNK_SIZE`] and [`MAX_CHUNK_SIZE`]. The value will be
    ///   automatically adjusted to the nearest power of 2.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    ///
    /// [`MIN_CHUNK_SIZE`]: crate::api::packing::packing_settings::MIN_CHUNK_SIZE
    /// [`MAX_CHUNK_SIZE`]: crate::api::packing::packing_settings::MAX_CHUNK_SIZE
    pub fn with_chunk_size(mut self, chunk_size: u32) -> Self {
        self.settings.chunk_size = chunk_size;
        self
    }

    /// Configures deduplication for chunked blocks in the archive.
    ///
    /// When enabled, the packer will detect and reuse duplicate chunks across
    /// large files, potentially reducing the archive size. This feature incurs
    /// a small memory overhead during packing.
    ///
    /// # Arguments
    ///
    /// * `enable` - Whether to enable chunked deduplication.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    pub fn with_chunked_deduplication(mut self, enable: bool) -> Self {
        self.settings.enable_chunked_deduplication = enable;
        self
    }

    /// Configures deduplication for SOLID blocks in the archive.
    ///
    /// When enabled, the packer will detect and reuse duplicate files within
    /// SOLID blocks. This feature has minimal performance impact during packing
    /// and can significantly reduce archive size when there are many duplicate files.
    ///
    /// # Arguments
    ///
    /// * `enable` - Whether to enable SOLID block deduplication.
    ///
    /// # Returns
    ///
    /// Returns self for method chaining.
    pub fn with_solid_deduplication(mut self, enable: bool) -> Self {
        self.settings.enable_solid_deduplication = enable;
        self
    }
}

impl Default for NxPackerBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameters used for adding a file to the archive.
#[derive(Debug, Clone)]
pub struct AddFileParams {
    /// Relative path of the file inside the archive. This path determines
    /// the file's location when the archive is extracted.
    pub relative_path: String,

    /// Preferred algorithm to compress the item with. This setting is only
    /// honored if [`SolidPreference::NoSolid`] is set in [`solid_type`].
    ///
    /// If no preference is specified (`NoPreference`), the archive's default
    /// compression algorithm will be used.
    pub compression_preference: CompressionPreference,

    /// Controls whether the file should be packed into a SOLID block
    /// or handled individually.
    pub solid_type: SolidPreference,
}

impl AddFileParams {
    /// Creates a new instance of `AddFileParams` with the specified relative path
    /// and default settings for compression and SOLID preferences.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - The path the file should have within the archive.
    ///
    /// # Returns
    ///
    /// A new `AddFileParams` instance with default compression and SOLID settings.
    pub fn new(relative_path: String) -> Self {
        Self {
            relative_path,
            compression_preference: CompressionPreference::NoPreference,
            solid_type: SolidPreference::Default,
        }
    }

    /// Creates a new instance of `AddFileParams` with all fields specified.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - The path the file should have within the archive.
    /// * `compression_preference` - The preferred compression algorithm.
    /// * `solid_type` - The SOLID block preference.
    ///
    /// # Returns
    ///
    /// A new `AddFileParams` instance with the specified settings.
    pub fn with_options(
        relative_path: String,
        compression_preference: CompressionPreference,
        solid_type: SolidPreference,
    ) -> Self {
        Self {
            relative_path,
            compression_preference,
            solid_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn can_configure_block_size() {
        let builder = NxPackerBuilder::new().with_block_size(65536);
        assert_eq!(builder.settings.block_size, 65536); // not yet sanitized
    }

    #[test]
    fn can_configure_chunk_size() {
        let builder = NxPackerBuilder::new().with_chunk_size(4194304);
        assert_eq!(builder.settings.chunk_size, 4194304); // Power of 2
    }

    #[test]
    fn can_configure_deduplication() {
        let builder = NxPackerBuilder::new()
            .with_chunked_deduplication(true)
            .with_solid_deduplication(false);

        assert!(builder.settings.enable_chunked_deduplication);
        assert!(!builder.settings.enable_solid_deduplication);
    }

    #[test]
    fn default_creates_new_instance() {
        let builder = NxPackerBuilder::default();
        assert!(!builder.settings.enable_chunked_deduplication);
        assert!(builder.settings.enable_solid_deduplication);
    }

    #[test]
    fn can_add_file_from_byte_slice() {
        let data = b"Hello, World!".to_vec();
        let mut builder = NxPackerBuilder::new();
        let options = AddFileParams {
            relative_path: String::from("test.txt"),
            compression_preference: CompressionPreference::NoPreference,
            solid_type: SolidPreference::Default,
        };

        builder.add_file_from_byte_slice(&data, options);

        assert_eq!(builder.files.len(), 1);
        let file = &builder.files[0];
        assert_eq!(file.relative_path(), "test.txt");
        assert_eq!(file.file_size(), 13);
    }

    #[test]
    fn can_add_file_from_boxed_slice() {
        let mut builder = NxPackerBuilder::new();
        let data = b"Hello, World!".to_vec();
        let options = AddFileParams {
            relative_path: String::from("test.txt"),
            compression_preference: CompressionPreference::NoPreference,
            solid_type: SolidPreference::Default,
        };

        builder.add_file_from_boxed_slice(data.into_boxed_slice(), options);

        assert_eq!(builder.files.len(), 1);
        let file = &builder.files[0];
        assert_eq!(file.relative_path(), "test.txt");
        assert_eq!(file.file_size(), 13);
    }

    #[test]
    fn can_add_file_from_stream() {
        let mut builder = NxPackerBuilder::new();
        let data = Cursor::new(b"Hello, World!".to_vec());
        let options = AddFileParams {
            relative_path: String::from("test.txt"),
            compression_preference: CompressionPreference::NoPreference,
            solid_type: SolidPreference::Default,
        };

        builder.add_file_from_stream(data, 13, options);

        assert_eq!(builder.files.len(), 1);
        let file = &builder.files[0];
        assert_eq!(file.relative_path(), "test.txt");
        assert_eq!(file.file_size(), 13);
    }
}

#[cfg(test)]
mod add_file_params_tests {
    use super::*;

    #[test]
    fn new_creates_with_defaults() {
        let params = AddFileParams::new("test.txt".into());
        assert_eq!(params.relative_path, "test.txt");
        assert!(matches!(
            params.compression_preference,
            CompressionPreference::NoPreference
        ));
        assert!(matches!(params.solid_type, SolidPreference::Default));
    }

    #[test]
    fn with_options_sets_all_fields() {
        let params = AddFileParams::with_options(
            "test.txt".into(),
            CompressionPreference::Lz4,
            SolidPreference::NoSolid,
        );

        assert_eq!(params.relative_path, "test.txt");
        assert!(matches!(
            params.compression_preference,
            CompressionPreference::Lz4
        ));
        assert!(matches!(params.solid_type, SolidPreference::NoSolid));
    }
}
