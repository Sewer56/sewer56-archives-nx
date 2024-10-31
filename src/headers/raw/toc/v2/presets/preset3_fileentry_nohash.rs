use crate::headers::managed::*;
use core::hash::Hash;
use endian_writer_derive::EndianWritable;
#[cfg(test)]
use fake::*;

/// Structure that represents the native serialized file entry
/// in the V2 Table of Contents format named 'Preset 3'.
/// This is the variant without file hashes.
///
/// See project documentation for more details.
#[repr(C, packed(8))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, EndianWritable)]
pub struct NativeFileEntryP3NoHash {
    /// [u32] Size of the file after decompression.
    pub decompressed_size: u32,

    /// [u16] Index of the file path in the stringpool.
    pub file_path_index: u16,

    /// [u16] Index of the block.
    pub block_index: u16,
}

impl From<FileEntry> for NativeFileEntryP3NoHash {
    fn from(entry: FileEntry) -> Self {
        NativeFileEntryP3NoHash {
            decompressed_size: entry.decompressed_size as u32,
            block_index: entry.first_block_index as u16,
            file_path_index: entry.file_path_index as u16,
        }
    }
}

impl From<NativeFileEntryP3NoHash> for FileEntry {
    fn from(value: NativeFileEntryP3NoHash) -> Self {
        FileEntry {
            hash: 0,
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
        assert_eq!(size_of::<NativeFileEntryP3NoHash>(), 8);
    }

    #[rstest]
    #[case::random_entry(Faker.fake())]
    fn can_copy_to_from_managed_entry(#[case] entry: NativeFileEntryP3NoHash) {
        let managed: FileEntry = entry.into();
        let new_entry: NativeFileEntryP3NoHash = managed.into();
        assert_eq!(new_entry, entry);
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
