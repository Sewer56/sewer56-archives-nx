// fileentryfieldsbits.rs
use super::Fef64TocHeader;

/// Structure that holds pre-calculated bit counts and masks for efficient file entry operations.
/// This is used for optimized reading of [FileEntry8] and [FileEntry16] structs.
///
/// All fields are public but immutable, allowing direct access for better performance
/// while maintaining safety.
///
/// [FileEntry8]: super::FileEntry8
/// [FileEntry16]: super::FileEntry16
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FileEntryFieldsBits {
    // Original bit counts
    pub block_count_bits: u8,
    pub file_count_bits: u8,
    pub decompressed_block_offset_bits: u8,

    // Pre-calculated (derived) values
    pub decompressed_size_bits: u8,

    // Pre-calculated masks
    pub block_count_mask: u64,
    pub file_count_mask: u64,
    pub decompressed_block_offset_mask: u64,
    pub decompressed_size_mask: u64,

    // Pre-calculated shifts
    pub file_count_shift: u32,
    pub decompressed_block_offset_shift: u32,
    pub decompressed_size_shift: u32,
}

impl FileEntryFieldsBits {
    pub fn new(
        block_count_bits: u8,
        file_count_bits: u8,
        decompressed_block_offset_bits: u8,
    ) -> Self {
        // Calculate used bits
        let used_bits = decompressed_block_offset_bits + file_count_bits + block_count_bits;

        // Calculate decompressed size bits
        let decompressed_size_bits = 64 - used_bits;

        // Calculate masks
        let block_count_mask = (1u64 << block_count_bits) - 1;
        let file_count_mask = (1u64 << file_count_bits) - 1;
        let decompressed_block_offset_mask = (1u64 << decompressed_block_offset_bits) - 1;
        let decompressed_size_mask = (1u64 << decompressed_size_bits) - 1;

        // Calculate shifts
        let file_count_shift = block_count_bits as u32;
        let decompressed_block_offset_shift = file_count_shift + file_count_bits as u32;
        let decompressed_size_shift = used_bits as u32;

        Self {
            block_count_bits,
            file_count_bits,
            decompressed_block_offset_bits,
            decompressed_size_bits,
            block_count_mask,
            file_count_mask,
            decompressed_block_offset_mask,
            decompressed_size_mask,
            file_count_shift,
            decompressed_block_offset_shift,
            decompressed_size_shift,
        }
    }

    // We can remove the getter methods since fields are now public
}

impl From<Fef64TocHeader> for FileEntryFieldsBits {
    fn from(header: Fef64TocHeader) -> Self {
        FileEntryFieldsBits::new(
            header.get_block_count_bits(),
            header.get_file_count_bits(),
            header.get_decompressed_block_offset_bits(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precalculated_values() {
        let fields = FileEntryFieldsBits::new(10, 10, 12);

        // Test basic calculations
        assert_eq!(fields.decompressed_size_bits, 32);

        // Test masks
        assert_eq!(fields.block_count_mask, 0x3FF); // 10 bits
        assert_eq!(fields.file_count_mask, 0x3FF); // 10 bits
        assert_eq!(fields.decompressed_block_offset_mask, 0xFFF); // 12 bits
        assert_eq!(fields.decompressed_size_mask, 0xFFFFFFFF); // 32 bits

        // Test shifts
        assert_eq!(fields.file_count_shift, 10);
        assert_eq!(fields.decompressed_block_offset_shift, 20);
        assert_eq!(fields.decompressed_size_shift, 32);
    }
}
