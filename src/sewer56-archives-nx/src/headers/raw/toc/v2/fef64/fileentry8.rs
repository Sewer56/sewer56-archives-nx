use super::FileEntryFieldsBits;
use crate::headers::managed::FileEntry;
use endian_writer::*;

/// Represents a 64-bit packed FileEntry using the Flexible Entry Format (without hash).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct FileEntry8 {
    data: u64,
}

impl FileEntry8 {
    /// Creates a new `FileEntry8` by packing the provided fields.
    ///
    /// # Arguments
    ///
    /// * `item_counts` - The bit counts for the fields.
    /// * `decompressed_size` - The decompressed size value.
    /// * `decompressed_block_offset` - The decompressed block offset value.
    /// * `file_path_index` - The file path index value.
    /// * `first_block_index` - The first block index value.
    pub fn new(
        item_counts: FileEntryFieldsBits,
        decompressed_size: u64,
        decompressed_block_offset: u64,
        file_path_index: u64,
        first_block_index: u64,
    ) -> Self {
        // Validate bit allocations in debug builds
        debug_assert_values_fit(
            &item_counts,
            decompressed_size,
            decompressed_block_offset,
            file_path_index,
            first_block_index,
        );

        // Pack the fields using pre-calculated masks and shifts
        let data = (decompressed_size & item_counts.decompressed_size_mask)
            << item_counts.decompressed_size_shift
            | (decompressed_block_offset & item_counts.decompressed_block_offset_mask)
                << item_counts.decompressed_block_offset_shift
            | (file_path_index & item_counts.file_count_mask) << item_counts.file_count_shift
            | (first_block_index & item_counts.block_count_mask);

        FileEntry8 { data }
    }

    /// Creates a new [FileEntry8] from a managed [FileEntry].
    ///
    /// # Arguments
    ///
    /// * `item_counts` - The bit counts for the fields.
    /// * `entry` - The managed representation of the file entry.
    #[inline]
    pub fn from_file_entry(item_counts: FileEntryFieldsBits, entry: &FileEntry) -> Self {
        Self::new(
            item_counts,
            entry.decompressed_size,
            entry.decompressed_block_offset as u64,
            entry.file_path_index as u64,
            entry.first_block_index as u64,
        )
    }

    #[inline(always)]
    pub fn decompressed_size(&self, counts: FileEntryFieldsBits) -> u64 {
        (self.data >> counts.decompressed_size_shift) & counts.decompressed_size_mask
    }

    #[inline(always)]
    pub fn decompressed_block_offset(&self, counts: FileEntryFieldsBits) -> u64 {
        (self.data >> counts.decompressed_block_offset_shift)
            & counts.decompressed_block_offset_mask
    }

    #[inline(always)]
    pub fn file_path_index(&self, counts: FileEntryFieldsBits) -> u64 {
        (self.data >> counts.file_count_shift) & counts.file_count_mask
    }

    #[inline(always)]
    pub fn first_block_index(&self, counts: FileEntryFieldsBits) -> u64 {
        self.data & counts.block_count_mask
    }

    /// Writes this file entry to the provided writer.
    ///
    /// # Arguments
    ///
    /// * `lewriter` - The writer to write to.
    #[inline(always)]
    pub fn to_writer(&self, lewriter: &mut LittleEndianWriter) {
        unsafe {
            lewriter.write_u64(self.data);
        }
    }

    /// Reads this managed file entry from data serialized as `NativeFileEntryV0`.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read from.
    #[inline(always)]
    pub fn from_reader(lereader: &mut LittleEndianReader) -> FileEntry8 {
        unsafe {
            let data = lereader.read_u64();
            FileEntry8 { data }
        }
    }

    /// Converts `FileEntry8` to `FileEntry`.
    ///
    /// # Arguments
    ///
    /// * `counts` - The bit counts used for extracting field values.
    ///
    /// # Returns
    ///
    /// A new `FileEntry` instance with the unpacked field values.
    #[inline(always)]
    pub fn to_file_entry(self, counts: FileEntryFieldsBits) -> FileEntry {
        FileEntry {
            decompressed_size: self.decompressed_size(counts),
            decompressed_block_offset: self.decompressed_block_offset(counts) as u32,
            file_path_index: self.file_path_index(counts) as u32,
            first_block_index: self.first_block_index(counts) as u32,
            hash: Default::default(), // No hash field in FileEntry8, so use default
        }
    }
}

fn debug_assert_values_fit(
    item_counts: &FileEntryFieldsBits,
    decompressed_size: u64,
    decompressed_block_offset: u64,
    file_path_index: u64,
    first_block_index: u64,
) {
    debug_assert!(decompressed_size <= item_counts.decompressed_size_mask);
    debug_assert!(decompressed_block_offset <= item_counts.decompressed_block_offset_mask);
    debug_assert!(file_path_index <= item_counts.file_count_mask);
    debug_assert!(first_block_index <= item_counts.block_count_mask);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fileentry8_size_is_correct() {
        assert_eq!(size_of::<FileEntry8>(), 8, "FileEntry8 should be 8 bytes");
    }

    #[test]
    fn fileentry8_packs_correctly() {
        let item_counts = FileEntryFieldsBits::new(10, 10, 12);

        let decompressed_size = 0xABCDE;
        let decompressed_block_offset = 0x123;
        let file_path_index = 0x3FF; // Max for 10 bits
        let first_block_index = 0x3FF; // Max for 10 bits

        let entry = FileEntry8::new(
            item_counts,
            decompressed_size,
            decompressed_block_offset,
            file_path_index,
            first_block_index,
        );

        // Use getter methods with ItemCounts for verification
        assert_eq!(
            entry.decompressed_size(item_counts),
            decompressed_size,
            "Decompressed size does not match"
        );
        assert_eq!(
            entry.decompressed_block_offset(item_counts),
            decompressed_block_offset,
            "Decompressed block offset does not match"
        );
        assert_eq!(
            entry.file_path_index(item_counts),
            file_path_index,
            "File path index does not match"
        );
        assert_eq!(
            entry.first_block_index(item_counts),
            first_block_index,
            "First block index does not match"
        );
    }

    #[test]
    fn fileentry8_to_fileentry_conversion_is_correct() {
        let item_counts = FileEntryFieldsBits::new(10, 10, 12);

        let decompressed_size = 0xABCDE;
        let decompressed_block_offset = 0x123;
        let file_path_index = 0x3FF; // Max for 10 bits
        let first_block_index = 0x3FF; // Max for 10 bits

        let entry8 = FileEntry8::new(
            item_counts,
            decompressed_size,
            decompressed_block_offset,
            file_path_index,
            first_block_index,
        );

        let entry = entry8.to_file_entry(item_counts);

        assert_eq!(
            entry.decompressed_size, decompressed_size,
            "Decompressed size does not match"
        );
        assert_eq!(
            entry.decompressed_block_offset, decompressed_block_offset as u32,
            "Decompressed block offset does not match"
        );
        assert_eq!(
            entry.file_path_index, file_path_index as u32,
            "File path index does not match"
        );
        assert_eq!(
            entry.first_block_index, first_block_index as u32,
            "First block index does not match"
        );
    }
}
