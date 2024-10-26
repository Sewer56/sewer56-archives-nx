use super::*;
use crate::headers::{managed::*, types::xxh3sum::XXH3sum};
use core::hash::Hash;
#[cfg(test)]
use fake::*;

/// Structure that represents the native serialized file entry
/// in the V2 Table of Contents format named 'Preset 2'.
///
/// See project documentation for more details.
#[repr(C, packed(8))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct NativeFileEntryP2 {
    /// [u64] Hash (XXH3) of the file described in this entry.
    pub hash: XXH3sum,

    /// [u64] Size of the file after decompression.
    pub decompressed_size: u64,

    offset_path_index_tuple: CommonOffsetPathIndexTuple,
}

#[coverage(off)] // Impl without coverage
impl NativeFileEntryP2 {
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

impl From<FileEntry> for NativeFileEntryP2 {
    fn from(entry: FileEntry) -> Self {
        NativeFileEntryP2 {
            hash: entry.hash.into(),
            decompressed_size: entry.decompressed_size,
            offset_path_index_tuple: CommonOffsetPathIndexTuple::new(
                entry.decompressed_block_offset,
                entry.file_path_index,
                entry.first_block_index,
            ),
        }
    }
}

impl From<NativeFileEntryP2> for FileEntry {
    fn from(value: NativeFileEntryP2) -> Self {
        FileEntry {
            hash: value.hash.into(),
            decompressed_size: value.decompressed_size,
            decompressed_block_offset: value.decompressed_block_offset(),
            file_path_index: value.file_path_index(),
            first_block_index: value.first_block_index(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn is_correct_size_bytes() {
        assert_eq!(size_of::<NativeFileEntryP2>(), 24);
    }

    #[rstest]
    #[case::random_entry(Faker.fake())]
    fn can_copy_to_from_managed_entry(#[case] entry: NativeFileEntryP2) {
        let managed: FileEntry = entry.into();
        let new_entry: NativeFileEntryP2 = managed.into();
        assert_eq!(new_entry, entry);
    }

    #[cfg(test)]
    impl Dummy<Faker> for NativeFileEntryP2 {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            NativeFileEntryP2 {
                hash: rng.gen::<u64>().into(),
                decompressed_size: rng.gen(),
                offset_path_index_tuple: Faker.fake(),
            }
        }
    }
}
