pub mod v1;
pub mod v2;

/// Represents the size of a compressed block following the header.
pub mod block_size;
/// Represents a file entry that was decoded from the Table of Contents.
pub mod file_entry;
/// Optimized functionality for dealing with file entries.
pub mod file_entry_intrinsics;
/// Allows for deserialization of the Table of Contents during the unpacking operation.
pub mod table_of_contents;

/// Prelude
pub use block_size::*;
pub use file_entry::*;
pub use table_of_contents::*;
