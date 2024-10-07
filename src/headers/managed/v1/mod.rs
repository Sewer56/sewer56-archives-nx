/// Allows for serialization of the Table of Contents during the packing operation.
pub mod table_of_contents_builder;
/// Allows for deserialization of the Table of Contents during the unpacking operation.
pub mod table_of_contents_reader;

/// Prelude
pub use table_of_contents_builder::*;
pub use table_of_contents_reader::*;
