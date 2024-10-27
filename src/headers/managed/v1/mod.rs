/// Optimized functionality for dealing with file entries.
pub mod file_entry_intrinsics;
/// Allows for serialization of the Table of Contents during the packing operation.
pub mod table_of_contents_builder;
/// Allows for deserialization of the Table of Contents during the unpacking operation.
pub mod table_of_contents_reader;

pub use file_entry_intrinsics::*;
/// Prelude
pub use table_of_contents_builder::*;
pub use table_of_contents_reader::*;
