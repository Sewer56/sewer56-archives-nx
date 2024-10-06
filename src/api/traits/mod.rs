/// Trait for items which can provide bytes corresponding to a file.
pub mod can_provide_file_data;
/// Used for items to with which format they would like to be compressed.
pub mod has_compression_preference;
/// Indicates the item has a file size. For data input into the packer.
pub mod has_file_size;
/// Indicates an item has a relative path. For data input into the packer.
pub mod has_relative_path;
/// Used for items to specify a preference on whether they'd prefer to be SOLIDly packed or not.
pub mod has_solid_type;

/// Prelude with re-exports
pub use can_provide_file_data::*;
pub use has_compression_preference::*;
pub use has_file_size::*;
pub use has_relative_path::*;
pub use has_solid_type::*;
