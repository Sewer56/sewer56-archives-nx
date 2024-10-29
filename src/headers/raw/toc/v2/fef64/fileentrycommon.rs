use super::Fef64TocHeader;

/// Structure that holds the bit counts used for fetching data from file entries.
/// This is used for easy reading of [FileEntry8] and [FileEntry16] structs.
///
/// [FileEntry8]: super::FileEntry8
/// [FileEntry16]: super::FileEntry16
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FileEntryFieldsBits {
    pub block_count_bits: u8,
    pub file_count_bits: u8,
    pub decompressed_block_offset_bits: u8,
}

impl FileEntryFieldsBits {
    pub fn new(
        block_count_bits: u8,
        file_count_bits: u8,
        decompressed_block_offset_bits: u8,
    ) -> Self {
        Self {
            block_count_bits,
            file_count_bits,
            decompressed_block_offset_bits,
        }
    }

    /// Calculates the number of bits allocated to decompressed_size.
    pub fn decompressed_size_bits(&self) -> u32 {
        64 - (self.decompressed_block_offset_bits as u32
            + self.file_count_bits as u32
            + self.block_count_bits as u32)
    }

    /// Calculates the total amount of used bits for everything that does not include
    /// the decompressed size; which derives from this.
    pub fn used_bits(&self) -> u32 {
        self.decompressed_block_offset_bits as u32
            + self.file_count_bits as u32
            + self.block_count_bits as u32
    }
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
