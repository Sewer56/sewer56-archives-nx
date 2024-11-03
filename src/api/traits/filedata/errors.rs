use alloc::string::String;
use lightweight_mmap::{handles::HandleOpenError, mmap::MmapError};
use thiserror_no_std::Error;

use crate::utilities::compression::NxDecompressionError;

/// Represents errors that can occur when providing file data.
///
/// This enum encapsulates various error types that might be encountered
/// while accessing or providing file data.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FileProviderError {
    #[error("Failed to acquire a lock by a file provider that requires it.")]
    FailedToAcquireLock(),

    #[error("Failed to seek stream to the start offset {0}")]
    FailedToSeekStream(u64),

    #[error("Failed to read {0} bytes from offset {1}")]
    FailedToReadFromStream(u64, u64),

    /// Error omitted from 3rd party integration
    #[error("Third party error: {0}")]
    ThirdPartyError(String),

    /// Failed to open file handle.
    #[error(transparent)]
    FileHandleOpenError(#[from] HandleOpenError),

    /// Failed to memory map a given file.
    #[error(transparent)]
    MmapError(#[from] MmapError),

    /// Failed to decompress Nx compressed data when sourcing from another Nx file.
    #[error("Failed to decompress Nx compressed data when sourcing from another Nx file.")]
    NxDecompressionError(#[from] NxDecompressionError),
}
