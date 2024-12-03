/// Trait for items which can provide bytes corresponding to a file.
pub mod can_provide_input_data;
/// Allows for specifying inputs and outputs for pack and extract operations.
pub mod filedata;
/// Used for items to with which format they would like to be compressed.
pub mod has_compression_preference;
/// Has a numbered dictionary index attached to the item.
pub mod has_dict_index;
/// Indicates the item has a file size. For data input into the packer.
pub mod has_file_size;
/// Indicates an item has a relative path. For data input into the packer.
pub mod has_relative_path;
/// Used for items to specify a preference on whether they'd prefer to be SOLIDly packed or not.
pub mod has_solid_type;
/// Used for reporting progress to external callers.
pub mod progress;

/// Prelude with re-exports
pub use can_provide_input_data::*;
pub use filedata::*;
pub use has_compression_preference::*;
pub use has_dict_index::*;
pub use has_file_size::*;
pub use has_relative_path::*;
pub use has_solid_type::*;
pub use progress::*;
