// Declarations
pub mod native_file_entry;
pub mod native_toc_header_v2;
pub mod native_toc_header_v3;

// Implementations of legacy (V1) header code.
pub mod v1;

pub use native_file_entry::*;
pub use native_toc_header_v2::*;
pub use native_toc_header_v3::*;

// Implementations of legacy (V2) header code.
pub mod v2;

/// Re-exports.
pub use v1::*;
pub use v2::*;
