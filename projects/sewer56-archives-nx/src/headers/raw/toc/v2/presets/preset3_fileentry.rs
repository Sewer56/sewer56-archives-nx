use crate::headers::{managed::*, types::xxh3sum::XXH3sum};
use core::hash::Hash;
use endian_writer_derive::EndianWritable;
#[cfg(test)]
use fake::*;

/// Structure that represents the native serialized file entry
/// in the V2 Table of Contents format named 'Preset 3'.
///
/// See project documentation for more details.
#[repr(C, packed(8))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, EndianWritable)]
pub struct NativeFileEntryP3 {
    /// [u64] Hash (XXH3) of the file described in this entry.
    pub hash: XXH3sum,

    /// [u32] Size of the file after decompression.
    pub decompressed_size: u32,

    /// [u16] Index of the file path in the stringpool.
    pub file_path_index: u16,

    /// [u16] Index of the block.
    pub block_index: u16,
}

impl From<FileEntry> for NativeFileEntryP3 {
    fn from(entry: FileEntry) -> Self {
        NativeFileEntryP3 {
            decompressed_size: entry.decompressed_size as u32,
            block_index: entry.first_block_index as u16,
            file_path_index: entry.file_path_index as u16,
            hash: entry.hash.into(),
        }
    }
}

impl From<NativeFileEntryP3> for FileEntry {
    fn from(value: NativeFileEntryP3) -> Self {
        FileEntry {
            hash: value.hash.0,
            decompressed_size: value.decompressed_size as u64,
            decompressed_block_offset: 0,
            file_path_index: value.file_path_index as u32,
            first_block_index: value.block_index as u32,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn is_correct_size_bytes() {
        assert_eq!(size_of::<NativeFileEntryP3>(), 16);
    }

    #[rstest]
    #[case::random_entry(Faker.fake())]
    fn can_copy_to_from_managed_entry(#[case] entry: NativeFileEntryP3) {
        let managed: FileEntry = entry.into();
        let new_entry: NativeFileEntryP3 = managed.into();
        assert_eq!(new_entry, entry);
    }

    #[cfg(test)]
    impl Dummy<Faker> for NativeFileEntryP3 {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            NativeFileEntryP3 {
                hash: rng.random::<u64>().into(),
                decompressed_size: rng.random(),
                block_index: rng.random(),
                file_path_index: rng.random(),
            }
        }
    }
}
