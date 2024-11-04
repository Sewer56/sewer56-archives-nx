use crate::api::traits::ReadOnlyFileData;

/// Implementation of [`ReadOnlyFileData`] backed by a byte slice.
pub struct SliceFileData<'a> {
    data: &'a [u8],
}

impl<'a> SliceFileData<'a> {
    /// Creates a new [SliceFileData] with the given byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> ReadOnlyFileData for SliceFileData<'a> {
    fn data(&self) -> &'a [u8] {
        self.data
    }
}
