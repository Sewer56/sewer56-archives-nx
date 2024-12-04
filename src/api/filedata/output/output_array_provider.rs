use crate::api::traits::filedata::*;
use crate::api::traits::FileOutputError;
use crate::prelude::*;
use crate::unsize_box2;
use core::cell::UnsafeCell;

/// Output data provider that writes data to an existing slice of data.
///
/// # Safety
///
/// This type is unsafe and violates Rust's Mutability rules.
/// Namely, it allows for multiple mutable references to the same underlying data.
///
/// This is okay, we want the data to be mutated from multiple threads at the same time,
/// so exposing multiple mutable references is by design.
pub struct OutputArrayProvider {
    /// The entry from the archive.
    entry: SmallFileEntry,

    /// The data buffer.
    data: UnsafeCell<Box<[u8]>>,
}

impl OutputArrayProvider {
    /// Initializes outputting a file to an array.
    ///
    /// # Arguments
    ///
    /// * `entry` - The entry from the archive.
    pub fn new(entry: SmallFileEntry) -> Result<Self, FileOutputError> {
        // SAFETY:
        // Note: On 32-bit systems, trying to extract a >2GiB (>4GiB with largeAddressAware on Windows)
        // filemay cause failures. In this case we use the fallible version of alloc (try_new_uninit_slice),
        // avoiding the panic that usually happens on allocation.

        Ok(Self {
            entry,
            data: unsafe {
                UnsafeCell::new(
                    Box::try_new_uninit_slice(entry.decompressed_size as usize)?.assume_init(),
                )
            },
        })
    }
}

impl OutputDataProvider for OutputArrayProvider {
    fn entry(&self) -> SmallFileEntry {
        self.entry
    }

    fn get_file_data<'a>(
        &'a self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn ReadWriteFileData + 'a>, FileProviderError> {
        let data = unsafe { &mut *self.data.get() };
        // All calls are done from within library and are assumed to be 'valid'
        debug_assert!(start + length <= data.len() as u64);

        Ok(unsize_box2!(Box::new(ArrayFileData::new(unsafe {
            data.get_unchecked_mut(start as usize..(start + length) as usize)
        }))))
    }
}

/// Implements FileData for array-based file data
struct ArrayFileData<'a> {
    data: &'a mut [u8],
}

impl<'a> ArrayFileData<'a> {
    fn new(data: &'a mut [u8]) -> Self {
        Self { data }
    }
}

impl ReadWriteFileData for ArrayFileData<'_> {
    fn data(&mut self) -> &mut [u8] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::traits::filedata::*;

    #[test]
    fn new_provider() {
        let entry = SmallFileEntry::new(100, 0, 0);
        let provider = OutputArrayProvider::new(entry).unwrap();
        assert_eq!(provider.entry(), entry);
    }

    #[test]
    fn new_provider_zero_size() {
        let entry = SmallFileEntry::new(0, 0, 0);
        let provider = OutputArrayProvider::new(entry).unwrap();
        assert_eq!(provider.entry(), entry);
    }

    #[test]
    fn get_full_range() {
        let entry = SmallFileEntry::new(100, 0, 0);
        let provider = OutputArrayProvider::new(entry).unwrap();
        let mut file_data = provider.get_file_data(0, 100).unwrap();
        assert_eq!(file_data.data().len(), 100);
    }

    #[test]
    fn get_partial_range() {
        let entry = SmallFileEntry::new(100, 0, 0);
        let provider = OutputArrayProvider::new(entry).unwrap();
        let mut file_data = provider.get_file_data(50, 30).unwrap();
        assert_eq!(file_data.data().len(), 30);
    }

    #[test]
    fn write_and_read_data() {
        let entry = SmallFileEntry::new(10, 0, 0);
        let provider = OutputArrayProvider::new(entry).unwrap();

        let test_data = [1, 2, 3, 4, 5];
        {
            let mut file_data = provider.get_file_data(0, 5).unwrap();
            file_data.data().copy_from_slice(&test_data);
        }

        {
            let mut file_data = provider.get_file_data(0, 5).unwrap();
            assert_eq!(file_data.data(), &test_data);
        }
    }

    #[test]
    fn multiple_range_writes() {
        let entry = SmallFileEntry::new(10, 0, 0);
        let provider = OutputArrayProvider::new(entry).unwrap();

        {
            let mut file_data = provider.get_file_data(0, 5).unwrap();
            file_data.data().copy_from_slice(&[1, 2, 3, 4, 5]);
        }

        {
            let mut file_data = provider.get_file_data(5, 5).unwrap();
            file_data.data().copy_from_slice(&[6, 7, 8, 9, 10]);
        }

        {
            let mut file_data = provider.get_file_data(0, 10).unwrap();
            assert_eq!(file_data.data(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        }
    }

    #[test]
    fn allocation_error() {
        // Note: This may fail on 128-bit systems one day.
        let entry = SmallFileEntry::new(u64::MAX, 0, 0);
        let result = OutputArrayProvider::new(entry);
        assert!(matches!(result, Err(FileOutputError::AllocError(_))));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "assertion failed")]
    fn invalid_range() {
        let entry = SmallFileEntry::new(10, 0, 0);
        let provider = OutputArrayProvider::new(entry).unwrap();
        let _file_data = provider.get_file_data(5, 10).unwrap();
    }

    #[test]
    fn verify_send() {
        fn assert_send<T: Send>() {}
        assert_send::<OutputArrayProvider>();
    }
}
