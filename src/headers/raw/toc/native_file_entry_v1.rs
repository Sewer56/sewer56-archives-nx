use super::*;
use crate::headers::managed::*;
use core::hash::Hash;

/// Structure that represents the native serialized file entry.
///
/// Remarks:
/// V1 represents [`TableOfContentsVersion::V1`](crate::headers::enums::table_of_contents_version::TableOfContentsVersion::V1).
#[repr(C, packed(8))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct NativeFileEntryV1 {
    /// [u64] Hash of the file described in this entry.
    pub hash: u64,

    /// [u64] Size of the file after decompression.
    pub decompressed_size: u64,

    offset_path_index_tuple: OffsetPathIndexTuple,
}

#[coverage(off)] // Impl without coverage
impl NativeFileEntryV1 {
    /// Size of item in bytes.
    pub(crate) const SIZE_BYTES: usize = 24;

    /// `u26` Gets the offset of the file inside the decompressed block.
    pub fn decompressed_block_offset(&self) -> u32 {
        self.offset_path_index_tuple.decompressed_block_offset()
    }

    /// `u26` Sets the offset of the file inside the decompressed block.
    pub fn set_decompressed_block_offset(&mut self, value: u32) {
        self.offset_path_index_tuple
            .set_decompressed_block_offset(value);
    }

    /// `u20` Gets the Index of the file path associated with this file in the StringPool.
    pub fn file_path_index(&self) -> u32 {
        self.offset_path_index_tuple.file_path_index()
    }

    /// `u20` Sets the Index of the file path associated with this file in the StringPool.
    pub fn set_file_path_index(&mut self, value: u32) {
        self.offset_path_index_tuple.set_file_path_index(value);
    }

    /// `u18` Gets the Index of the first block associated with this file.
    pub fn first_block_index(&self) -> u32 {
        self.offset_path_index_tuple.first_block_index()
    }

    /// `u18` Sets the Index of the first block associated with this file.
    pub fn set_first_block_index(&mut self, value: u32) {
        self.offset_path_index_tuple.set_first_block_index(value);
    }
}

impl NativeFileEntry for NativeFileEntryV1 {
    fn copy_from(&mut self, entry: &FileEntry) {
        self.hash = entry.hash;
        self.decompressed_size = entry.decompressed_size;
        self.offset_path_index_tuple = OffsetPathIndexTuple::new(
            entry.decompressed_block_offset,
            entry.file_path_index,
            entry.first_block_index,
        );
    }

    fn copy_to(&self, entry: &mut FileEntry) {
        entry.hash = self.hash;
        entry.decompressed_size = self.decompressed_size;
        self.offset_path_index_tuple.copy_to(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::*;
    use rstest::rstest;

    #[test]
    fn is_correct_size_bytes() {
        assert_eq!(
            size_of::<NativeFileEntryV1>(),
            NativeFileEntryV1::SIZE_BYTES
        );
    }

    #[rstest]
    #[case::random_entry(Faker.fake())]
    fn can_copy_to_from_managed_entry(#[case] entry: NativeFileEntryV1) {
        use crate::headers::raw::toc::native_file_entry_v0::tests::test_copy_to_and_from_managed_entry;
        test_copy_to_and_from_managed_entry(&entry);
    }

    impl Dummy<Faker> for NativeFileEntryV1 {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            NativeFileEntryV1 {
                hash: rng.gen(),
                decompressed_size: rng.gen(),
                offset_path_index_tuple: Faker.fake(),
            }
        }
    }
}
