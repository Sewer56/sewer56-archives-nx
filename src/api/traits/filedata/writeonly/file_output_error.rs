use lightweight_mmap::handles::HandleOpenError;
use std::alloc::AllocError;
use thiserror_no_std::Error;

/// Represents errors that can occur when providing outputs for files.
///
/// These errors correspond to built-in implementations of the [`OutputDataProvider`] trait.
///
/// [`OutputDataProvider`]: crate::api::traits::filedata::OutputDataProvider
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FileOutputError {
    #[error(transparent)]
    AllocError(#[from] AllocError),

    /// Failed to open file handle.
    #[error(transparent)]
    FileHandleOpenError(#[from] HandleOpenError),
}
