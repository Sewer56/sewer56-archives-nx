use crate::api::{enums::*, filedata::*, traits::*};

/// Represents a file that will be packed into an Nx archive.
pub struct PackerFile<'a> {
    /// Relative path within the archive
    relative_path: String,

    /// Size of the file in bytes
    file_size: u64,

    /// Provider for accessing the file's contents
    data_provider: Box<dyn InputDataProvider + Send + Sync + 'a>,

    /// How this file should be compressed
    compression_preference: CompressionPreference,

    /// Whether this file should be in a SOLID block
    solid_preference: SolidPreference,
}

/// Manual implementation of Debug, to skip InputDataProvider
impl std::fmt::Debug for PackerFile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackerFile")
            .field("relative_path", &self.relative_path)
            .field("file_size", &self.file_size)
            .field("compression_preference", &self.compression_preference)
            .field("solid_preference", &self.solid_preference)
            .finish()
    }
}

impl<'a> PackerFile<'a> {
    /// Creates a new PackerFile instance.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Path the file should have within the archive
    /// * `file_size` - Size of the file in bytes
    /// * `provider` - Provider for reading the file's contents
    pub fn new(
        relative_path: String,
        file_size: u64,
        provider: Box<dyn InputDataProvider + Send + Sync + 'a>,
    ) -> Self {
        Self {
            relative_path,
            file_size,
            data_provider: provider,
            compression_preference: CompressionPreference::NoPreference,
            solid_preference: SolidPreference::Default,
        }
    }

    /// Creates a new PackerFile from a path, automatically creating the provider.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the file on disk
    /// * `relative_path` - Path the file should have within the archive
    /// * `file_size` - Size of the file in bytes
    ///
    /// # Returns
    ///
    /// A Result containing either the new PackerFile or an error if the provider couldn't be created
    pub fn from_file_path(
        source_path: &str,
        relative_path: String,
        file_size: u64,
    ) -> Result<Self, FileProviderError> {
        let provider = Box::new(FromFilePathProvider::new(source_path)?);
        Ok(Self::new(relative_path, file_size, provider))
    }

    /// Creates a new PackerFile from a path, automatically creating the provider.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the file on disk
    /// * `relative_path` - Path the file should have within the archive
    ///
    /// # Returns
    ///
    /// A Result containing either the new PackerFile or an error if the provider couldn't be created
    pub fn from_file_path_with_unknown_size(
        source_path: &str,
        relative_path: String,
    ) -> Result<Self, FileProviderError> {
        let provider = Box::new(FromFilePathProvider::new(source_path)?);
        Ok(Self::new(
            relative_path,
            provider.file_size()? as u64,
            provider,
        ))
    }

    /// Sets the compression preference for this file
    pub fn with_compression(mut self, preference: CompressionPreference) -> Self {
        self.compression_preference = preference;
        self
    }

    /// Sets the SOLID block preference for this file
    pub fn with_solid(mut self, preference: SolidPreference) -> Self {
        self.solid_preference = preference;
        self
    }
}

impl HasFileSize for PackerFile<'_> {
    fn file_size(&self) -> u64 {
        self.file_size
    }
}

impl HasRelativePath for PackerFile<'_> {
    fn relative_path(&self) -> &str {
        &self.relative_path
    }
}

impl HasCompressionPreference for PackerFile<'_> {
    fn compression_preference(&self) -> CompressionPreference {
        self.compression_preference
    }
}

impl HasSolidType for PackerFile<'_> {
    fn solid_type(&self) -> SolidPreference {
        self.solid_preference
    }

    fn set_solid_type(&mut self, preference: SolidPreference) {
        self.solid_preference = preference;
    }
}

impl CanProvideInputData for PackerFile<'_> {
    fn input_data_provider(&self) -> &dyn InputDataProvider {
        &*self.data_provider
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[cfg_attr(miri, ignore)] // uses I/O, unsupported
    fn can_create_from_file_path() {
        // Create a temporary test file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        temp_file.flush().unwrap();

        let file = PackerFile::from_file_path(
            temp_file.path().to_str().unwrap(),
            "test.txt".to_string(),
            12, // "test content\n".len()
        )
        .unwrap();

        assert_eq!(file.relative_path(), "test.txt");
        assert_eq!(file.file_size(), 12);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn can_set_preferences() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test").unwrap();
        temp_file.flush().unwrap();

        let file = PackerFile::from_file_path(
            temp_file.path().to_str().unwrap(),
            "test.txt".to_string(),
            5,
        )
        .unwrap()
        .with_compression(CompressionPreference::Lz4)
        .with_solid(SolidPreference::NoSolid);

        assert_eq!(file.compression_preference(), CompressionPreference::Lz4);
        assert_eq!(file.solid_type(), SolidPreference::NoSolid);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn handles_invalid_path() {
        let file_name = "test.txt".to_string();
        let result = PackerFile::from_file_path("nonexistent.txt", file_name, 0);

        assert!(result.is_err());
    }
}
