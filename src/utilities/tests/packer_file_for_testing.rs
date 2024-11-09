use crate::api::{
    enums::*,
    traits::{
        can_provide_input_data::CanProvideInputData,
        has_compression_preference::HasCompressionPreference, has_file_size::HasFileSize,
        has_relative_path::HasRelativePath, has_solid_type::HasSolidType, InputDataProvider,
    },
};
use alloc::rc::Rc;

#[derive(Clone)]
pub struct PackerFileForTesting {
    relative_path: String,
    file_size: u64,
    solid_preference: SolidPreference,
    compression_preference: CompressionPreference,
}

impl PackerFileForTesting {
    pub fn new(relative_path: &str, file_size: u64) -> Self {
        Self {
            relative_path: relative_path.to_string(),
            file_size,
            compression_preference: CompressionPreference::NoPreference,
            solid_preference: SolidPreference::Default,
        }
    }

    pub fn new_rc(relative_path: &str, file_size: u64) -> Rc<Self> {
        Rc::new(Self::new(relative_path, file_size))
    }
}

impl HasFileSize for PackerFileForTesting {
    fn file_size(&self) -> u64 {
        self.file_size
    }
}

impl HasRelativePath for PackerFileForTesting {
    fn relative_path(&self) -> &str {
        &self.relative_path
    }
}

impl HasSolidType for PackerFileForTesting {
    fn solid_type(&self) -> SolidPreference {
        self.solid_preference
    }
}

impl HasCompressionPreference for PackerFileForTesting {
    fn compression_preference(&self) -> CompressionPreference {
        self.compression_preference
    }
}

impl CanProvideInputData for PackerFileForTesting {
    fn input_data_provider(&self) -> &dyn InputDataProvider {
        todo!()
    }
}
