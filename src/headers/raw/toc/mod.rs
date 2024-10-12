// Declarations
pub mod native_file_entry;

// Implementations of legacy (V1) header code.
pub mod v1;

pub use native_file_entry::*;

// Implementations of legacy (V2) header code.
pub mod v2;

/// Re-exports.
pub use v1::*;
pub use v2::*;
