use crate::headers::{managed::*, raw::toc::NativeFileEntry};
use core::hash::Hash;
#[cfg(test)]
use fake::*;

/// Structure that represents the native serialized file entry
/// in the V2 Table of Contents format named 'Preset 3'.
/// This is the variant without file hashes.
///
/// See project documentation for more details.
#[repr(C, packed(8))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct NativeFileEntryP3NoHash {
    /// [u32] Size of the file after decompression.
    pub decompressed_size: u32,

    /// [u16] Index of the file path in the stringpool.
    pub file_path_index: u16,

    /// [u16] Index of the block.
    pub block_index: u16,
}

impl NativeFileEntry for NativeFileEntryP3NoHash {
    fn copy_from(&mut self, entry: &FileEntry) {
        self.decompressed_size = entry.decompressed_size as u32;
        self.file_path_index = entry.file_path_index as u16;
        self.block_index = entry.first_block_index as u16;
    }

    fn copy_to(&self, entry: &mut FileEntry) {
        entry.decompressed_size = self.decompressed_size as u64;
        entry.file_path_index = self.file_path_index as u32;
        entry.first_block_index = self.block_index as u32;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use core::fmt::Debug;
    use rstest::rstest;

    #[test]
    fn is_correct_size_bytes() {
        assert_eq!(size_of::<NativeFileEntryP3NoHash>(), 8);
    }

    #[rstest]
    #[case::random_entry(Faker.fake())]
    fn can_copy_to_from_managed_entry(#[case] entry: NativeFileEntryP3NoHash) {
        test_copy_to_and_from_managed_entry(&entry);
    }

    pub(crate) fn test_copy_to_and_from_managed_entry<
        T: NativeFileEntry + PartialEq + Default + Debug,
    >(
        entry: &T,
    ) {
        let mut new_entry = T::default();
        let mut managed = FileEntry::default();

        // Do a round trip copy, and compare new_entry with old_entry.
        // If both are equal, the copy operation is successful.
        entry.copy_to(&mut managed);
        new_entry.copy_from(&managed);

        assert_eq!(&new_entry, entry);
    }

    #[cfg(test)]
    impl Dummy<Faker> for NativeFileEntryP3NoHash {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            NativeFileEntryP3NoHash {
                decompressed_size: rng.gen(),
                block_index: rng.gen(),
                file_path_index: rng.gen(),
            }
        }
    }
}
