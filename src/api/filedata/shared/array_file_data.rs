use crate::api::traits::FileData;

/// Implementation of [`FileData`] backed by a byte slice.
pub struct SliceFileData<'a> {
    data: &'a [u8],
}

impl<'a> SliceFileData<'a> {
    /// Creates a new [SliceFileData] with the given byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> FileData for SliceFileData<'a> {
    fn data(&self) -> &'a [u8] {
        self.data
    }
}
