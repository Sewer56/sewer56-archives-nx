use super::*;
use crate::headers::managed::*;
use core::hash::Hash;
use endian_writer_derive::EndianWritable;
#[cfg(test)]
use fake::*;

/// Structure that represents the native serialized file entry
/// in the V2 Table of Contents format named 'Preset 1'.
///
/// See project documentation for more details.
#[repr(C, packed(4))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, EndianWritable)]
pub struct NativeFileEntryP1 {
    /// [u32] Size of the file after decompression.
    pub decompressed_size: u32,

    offset_path_index_tuple: CommonOffsetPathIndexTuple,
}

#[coverage(off)] // Impl without coverage
impl NativeFileEntryP1 {
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

impl From<FileEntry> for NativeFileEntryP1 {
    fn from(entry: FileEntry) -> Self {
        NativeFileEntryP1 {
            decompressed_size: entry.decompressed_size as u32,
            offset_path_index_tuple: CommonOffsetPathIndexTuple::new(
                entry.decompressed_block_offset,
                entry.file_path_index,
                entry.first_block_index,
            ),
        }
    }
}

impl From<NativeFileEntryP1> for FileEntry {
    fn from(value: NativeFileEntryP1) -> Self {
        FileEntry {
            hash: 0,
            decompressed_size: value.decompressed_size as u64,
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
        assert_eq!(size_of::<NativeFileEntryP1>(), 12);
    }

    #[rstest]
    #[case::random_entry(Faker.fake())]
    fn can_copy_to_from_managed_entry(#[case] entry: NativeFileEntryP1) {
        let managed: FileEntry = entry.into();
        let new_entry: NativeFileEntryP1 = managed.into();
        assert_eq!(new_entry, entry);
    }

    #[cfg(test)]
    impl Dummy<Faker> for NativeFileEntryP1 {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            NativeFileEntryP1 {
                decompressed_size: rng.gen(),
                offset_path_index_tuple: Faker.fake(),
            }
        }
    }
}
