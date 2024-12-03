use crate::api::traits::Progress;
use std::io::{Seek, Write};
use thiserror_no_std::Error;

use super::{
    chunked_deduplication_state::ChunkedDeduplicationState,
    solid_deduplication_state::SolidDeduplicationState,
};

/// Holds the mutable state of the packer.
///
/// # Safety
///
/// This struct is not thread-safe and stores mutable state tied to the packer,
/// such as progress reporting and deduplication states.
pub struct PackingState<'a, W: Write + Seek> {
    /// The output to which the packer writes data to.
    pub output: W,

    /// Reports progress back to the process.
    /// Reported values are between 0.0 and 1.0.
    pub progress: &'a dyn Progress,

    /// If not `None`, chunked files are deduplicated.
    /// Chunked deduplication incurs a small amount of overhead for each file.
    pub chunked_deduplication_state: Option<ChunkedDeduplicationState>,

    /// If not `None`, files are deduplicated.
    /// Solid deduplication incurs a small amount of overhead for each block.
    pub solid_deduplication_state: Option<SolidDeduplicationState>,
}

impl<'a, W: Write + Seek> PackingState<'a, W> {
    /// Creates a new `PackingState` with default values.
    pub fn new(progress: &'a dyn Progress, output: W) -> Self {
        PackingState {
            output,
            progress,
            chunked_deduplication_state: None,
            solid_deduplication_state: Some(SolidDeduplicationState::new()),
        }
    }
}

/// Errors that can occur during deduplication operations
#[derive(Error, Debug)]
pub enum DeduplicationError {
    #[error("Failed to acquire read lock")]
    ReadLockError,
    #[error("Failed to acquire write lock")]
    WriteLockError,
}
