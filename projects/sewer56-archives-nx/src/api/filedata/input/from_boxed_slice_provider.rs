use crate::api::{filedata::SliceFileData, traits::*};
use crate::{prelude::*, unsize_box2};

/// Provides file data from an in-memory byte array.
pub struct FromBoxedSliceProvider {
    data: Box<[u8]>,
}

impl FromBoxedSliceProvider {
    /// Creates a new [`FromArrayProvider`] with the given boxed byte slice.
    pub fn new(data: Box<[u8]>) -> Self {
        Self { data }
    }
}

impl InputDataProvider for FromBoxedSliceProvider {
    fn get_file_data<'a>(
        &'a self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn ReadOnlyFileData + 'a>, FileProviderError> {
        let start = start as usize;
        let length = length as usize;

        // SAFETY: We know `start` and `length` fall within bounds of `self.data`
        //         The calls to `get_file_data` are done by the library and are thus
        //         assumed to be 'safe'/'trusted'.
        debug_assert!(start + length <= self.data.len());
        let slice = unsafe { self.data.get_unchecked(start..start + length) };
        Ok(unsize_box2!(Box::new(SliceFileData::new(slice))))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        api::{filedata::FromBoxedSliceProvider, traits::InputDataProvider},
        prelude::*,
        unsize_box2,
    };

    #[test]
    fn from_array_provider_has_valid_range() {
        let data: Box<[u8]> = unsize_box2!(Box::new([10, 20, 30, 40, 50]));
        let provider = FromBoxedSliceProvider::new(data);

        let file_data = provider.get_file_data(1, 3).unwrap();
        assert_eq!(file_data.data(), &[20, 30, 40]);
    }
}
