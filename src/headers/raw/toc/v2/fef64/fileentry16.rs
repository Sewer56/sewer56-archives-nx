use super::ItemCounts;
use crate::{headers::types::xxh3sum::XXH3sum, utilities::math::ToBitmask};

/// Represents a 128-bit packed FileEntry using Flexible Entry Format (with hash).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct FileEntry16 {
    hash: XXH3sum,
    data: u64,
}

impl FileEntry16 {
    /// Creates a new `FileEntry16` by packing the provided fields, including the hash.
    ///
    /// # Arguments
    ///
    /// * `item_counts` - The bit counts for the fields.
    /// * `hash` - The hash value (XXH3sum).
    /// * `decompressed_size` - The decompressed size value.
    /// * `decompressed_block_offset` - The decompressed block offset value.
    /// * `file_path_index` - The file path index value.
    /// * `first_block_index` - The first block index value.
    pub fn new(
        item_counts: ItemCounts,
        hash: XXH3sum,
        decompressed_size: u64,
        decompressed_block_offset: u64,
        file_path_index: u64,
        first_block_index: u64,
    ) -> Self {
        // Validate bit allocations in debug builds
        debug_assert!(item_counts.used_bits() < 64);

        // Calculate bits allocated for decompressed_size
        let decompressed_size_bits = item_counts.decompressed_size_bits();

        // Validate that values fit within their bit allocations in debug builds
        debug_assert!(decompressed_size < (1u64 << decompressed_size_bits),);
        debug_assert!(
            decompressed_block_offset < (1u64 << item_counts.decompressed_block_offset_bits),
        );
        debug_assert!(file_path_index < (1u64 << item_counts.file_count_bits),);
        debug_assert!(first_block_index < (1u64 << item_counts.block_count_bits),);

        // Create bitmasks using the ToBitmask trait
        let decompressed_size_mask = decompressed_size_bits.to_bitmask();
        let decompressed_block_offset_mask =
            item_counts.decompressed_block_offset_bits.to_bitmask();
        let file_path_index_mask = item_counts.file_count_bits.to_bitmask();
        let first_block_index_mask = item_counts.block_count_bits.to_bitmask();

        // Pack the fields into a single u64 with decompressed_size in upper bits
        let data = (decompressed_size & decompressed_size_mask) << item_counts.used_bits()
            | (decompressed_block_offset & decompressed_block_offset_mask)
                << (item_counts.file_count_bits + item_counts.block_count_bits)
            | (file_path_index & file_path_index_mask) << item_counts.block_count_bits
            | (first_block_index & first_block_index_mask);

        FileEntry16 { hash, data }
    }

    /// Returns the hash as a XXH3sum.
    pub fn hash(&self) -> XXH3sum {
        self.hash
    }

    /// Returns the packed data as a u64.
    pub fn data(&self) -> u64 {
        self.data
    }

    /// Returns the decompressed size.
    pub fn decompressed_size(&self, counts: &ItemCounts) -> u64 {
        let decompressed_size_bits = counts.decompressed_size_bits();
        (self.data >> counts.used_bits()) & decompressed_size_bits.to_bitmask()
    }

    /// Returns the decompressed block offset.
    pub fn decompressed_block_offset(&self, counts: &ItemCounts) -> u64 {
        (self.data >> (counts.file_count_bits + counts.block_count_bits))
            & counts.decompressed_block_offset_bits.to_bitmask()
    }

    /// Returns the file path index.
    pub fn file_path_index(&self, counts: &ItemCounts) -> u64 {
        (self.data >> counts.block_count_bits) & counts.file_count_bits.to_bitmask()
    }

    /// Returns the first block index.
    pub fn first_block_index(&self, counts: &ItemCounts) -> u64 {
        self.data & counts.block_count_bits.to_bitmask()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fileentry16_size_is_correct() {
        assert_eq!(
            size_of::<FileEntry16>(),
            16,
            "FileEntry16 should be 16 bytes"
        );
    }

    #[test]
    fn fileentry16_packs_correctly() {
        let item_counts = ItemCounts {
            block_count_bits: 10,
            file_count_bits: 10,
            decompressed_block_offset_bits: 12,
        };

        let hash = XXH3sum(0xDEADBEEFDEADBEEF);
        let decompressed_size = 0xABCDE;
        let decompressed_block_offset = 0x123;
        let file_path_index = 0x3FF; // Max for 10 bits
        let first_block_index = 0x3FF; // Max for 10 bits

        let entry = FileEntry16::new(
            item_counts,
            hash,
            decompressed_size,
            decompressed_block_offset,
            file_path_index,
            first_block_index,
        );

        assert_eq!(entry.hash().0, 0xDEADBEEFDEADBEEF, "Hash does not match");
        assert_eq!(
            entry.decompressed_size(&item_counts),
            decompressed_size,
            "Decompressed size does not match"
        );
        assert_eq!(
            entry.decompressed_block_offset(&item_counts),
            decompressed_block_offset,
            "Decompressed block offset does not match"
        );
        assert_eq!(
            entry.file_path_index(&item_counts),
            file_path_index,
            "File path index does not match"
        );
        assert_eq!(
            entry.first_block_index(&item_counts),
            first_block_index,
            "First block index does not match"
        );
    }
}
