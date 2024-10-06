/// Represents the size of a compressed block following the header.
pub mod block_size;
/// Represents a file entry that was decoded from the Table of Contents.
pub mod file_entry;
/// Optimized functionality for dealing with file entries.
pub mod file_entry_intrinsics;
/// Allows for serialization of the Table of Contents during the packing operation.
pub mod table_of_contents_builder;
/// Allows for deserialization of the Table of Contents during the unpacking operation.
pub mod table_of_contents_reader;

/// Prelude
pub use block_size::*;
pub use file_entry::*;
pub use table_of_contents_builder::*;
pub use table_of_contents_reader::*;
