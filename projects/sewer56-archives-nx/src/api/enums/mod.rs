/// Allows you to specify how the data should be compressed.
pub mod compression_preference;
/// Allows you to specify whether a given file should be SOLID or not.
pub mod solid_preference;

/// Prelude
pub use compression_preference::*;
pub use solid_preference::*;
