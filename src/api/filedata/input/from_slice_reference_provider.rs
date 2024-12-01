use crate::api::{filedata::SliceFileData, traits::*};
use alloc::boxed::Box;

/// Provides file data from an in-memory byte array.
pub struct FromSliceReferenceProvider<'a> {
    data: &'a [u8],
}

impl<'a> FromSliceReferenceProvider<'a> {
    /// Creates a new [`FromArrayProvider`] with the given slice reference.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl InputDataProvider for FromSliceReferenceProvider<'_> {
    fn get_file_data<'b>(
        &'b self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn ReadOnlyFileData + 'b>, FileProviderError> {
        let start = start as usize;
        let length = length as usize;

        // SAFETY: We know `start` and `length` fall within bounds of `self.data`
        //         The calls to `get_file_data` are done by the library and are thus
        //         assumed to be 'safe'/'trusted'.
        debug_assert!(start + length <= self.data.len());
        let slice = unsafe { self.data.get_unchecked(start..start + length) };
        Ok(Box::new(SliceFileData::new(slice)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_array_provider_has_valid_range() {
        let data = [10, 20, 30, 40, 50];
        let provider = FromSliceReferenceProvider::new(&data);

        let file_data = provider.get_file_data(1, 3).unwrap();
        assert_eq!(file_data.data(), &[20, 30, 40]);
    }
}
