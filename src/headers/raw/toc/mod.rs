// Declarations
pub mod native_file_entry;
pub mod native_toc_header_v2;
pub mod native_toc_header_v3;

// Implementations of legacy (V1) header code.
pub mod v1;

pub use native_file_entry::*;
pub use native_toc_header_v2::*;
pub use native_toc_header_v3::*;

/// Provides re-exports, use with use `prelude::*`
pub use v1::*;
