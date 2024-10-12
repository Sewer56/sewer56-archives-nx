use super::*;
use crate::headers::{managed::*, raw::toc::NativeFileEntry, types::xxh3sum::XXH3sum};
use core::hash::Hash;
#[cfg(test)]
use fake::*;

/// Structure that represents the native serialized file entry
/// in the V2 Table of Contents format named 'Preset 0'.
///
/// See project documentation for more details.
#[repr(C, packed(4))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct NativeFileEntryP0 {
    /// [u64] Hash (XXH3) of the file described in this entry.
    pub hash: XXH3sum,

    /// [u32] Size of the file after decompression.
    pub decompressed_size: u32,

    offset_path_index_tuple: CommonOffsetPathIndexTuple,
}

#[coverage(off)] // Impl without coverage
impl NativeFileEntryP0 {
    /// `u24` Offset of the file inside the decompressed block.
    pub fn decompressed_block_offset(&self) -> u32 {
        { self.offset_path_index_tuple }.decompressed_block_offset()
    }

    /// `u24` Offset of the file inside the decompressed block.
    pub fn set_decompressed_block_offset(&mut self, value: u32) {
        let mut tuple = self.offset_path_index_tuple;
        tuple.set_decompressed_block_offset(value);
        self.offset_path_index_tuple = tuple;
    }

    /// `u18` Index of the file path associated with this file in the StringPool.
    pub fn file_path_index(&self) -> u32 {
        { self.offset_path_index_tuple }.file_path_index()
    }

    /// `u18` Index of the file path associated with this file in the StringPool.
    pub fn set_file_path_index(&mut self, value: u32) {
        let mut tuple = self.offset_path_index_tuple;
        tuple.set_file_path_index(value);
        self.offset_path_index_tuple = tuple;
    }

    /// `u22` Index of the first block associated with this file.
    pub fn first_block_index(&self) -> u32 {
        { self.offset_path_index_tuple }.first_block_index()
    }

    /// `u22` Index of the first block associated with this file.
    pub fn set_first_block_index(&mut self, value: u32) {
        let mut tuple = self.offset_path_index_tuple;
        tuple.set_first_block_index(value);
        self.offset_path_index_tuple = tuple;
    }
}

impl NativeFileEntry for NativeFileEntryP0 {
    fn copy_from(&mut self, entry: &FileEntry) {
        self.hash.0 = entry.hash;
        self.decompressed_size = entry.decompressed_size as u32;
        self.offset_path_index_tuple = CommonOffsetPathIndexTuple::new(
            entry.decompressed_block_offset,
            entry.file_path_index,
            entry.first_block_index,
        );
    }

    fn copy_to(&self, entry: &mut FileEntry) {
        entry.hash = self.hash.0;
        entry.decompressed_size = self.decompressed_size as u64;
        { self.offset_path_index_tuple }.copy_to(entry);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use core::fmt::Debug;
    use rstest::rstest;

    #[test]
    fn is_correct_size_bytes() {
        assert_eq!(size_of::<NativeFileEntryP0>(), 20);
    }

    #[rstest]
    #[case::random_entry(Faker.fake())]
    fn can_copy_to_from_managed_entry(#[case] entry: NativeFileEntryP0) {
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
    impl Dummy<Faker> for NativeFileEntryP0 {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            NativeFileEntryP0 {
                hash: rng.gen::<u64>().into(),
                decompressed_size: rng.gen(),
                offset_path_index_tuple: Faker.fake(),
            }
        }
    }
}
