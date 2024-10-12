/// Structure that holds the bit counts used for fetching data from file entries.
/// This is used for easy reading of [FileEntry8] and [FileEntry16] structs.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ItemCounts {
    pub block_count_bits: u32,
    pub file_count_bits: u32,
    pub decompressed_block_offset_bits: u32,
}

impl ItemCounts {
    /// Calculates the number of bits allocated to decompressed_size.
    pub fn decompressed_size_bits(&self) -> u32 {
        64 - (self.decompressed_block_offset_bits + self.file_count_bits + self.block_count_bits)
    }

    /// Calculates the total amount of used bits for everything that does not include
    /// the decompressed size; which derives from this.
    pub fn used_bits(&self) -> u32 {
        self.decompressed_block_offset_bits + self.file_count_bits + self.block_count_bits
    }
}
