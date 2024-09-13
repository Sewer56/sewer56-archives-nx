/// Holds the mutable state of the packer.
///
/// # Safety
///
/// This struct is not thread-safe and stores mutable state tied to the packer,
/// such as progress reporting and deduplication states.
pub struct PackingState<'a> {
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

impl<'a> PackingState<'a> {
    /// Creates a new `PackingState` with default values.
    pub fn new(progress: &'a dyn Progress) -> Self {
        PackingState {
            progress,
            chunked_deduplication_state: None,
            solid_deduplication_state: Some(SolidDeduplicationState::new()),
        }
    }

    /// Resets the deduplication states.
    pub fn reset_states(&mut self) {
        if let Some(ref mut state) = self.chunked_deduplication_state {
            state.reset();
        }
        if let Some(ref mut state) = self.solid_deduplication_state {
            state.reset();
        }
    }
}
