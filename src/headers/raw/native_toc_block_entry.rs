use bitfield::bitfield;

use crate::api::enums::compression_preference::CompressionPreference;

bitfield! {
    /// Native 'block entry' in the 'Table of Contents'
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct NativeTocBlockEntry(u32);
    impl Debug;
    u32;

    /// `u29` The compressed size of the block.
    pub compressed_block_size, set_compressed_block_size: 31, 3;

    // Keep the raw getter/setter, but make them private
    compression_raw, set_compression_raw: 2, 0;
}

impl NativeTocBlockEntry {
    /// Create a new NativeTocBlockEntry
    pub fn new(compressed_block_size: u32, compression: CompressionPreference) -> Self {
        let mut header = NativeTocBlockEntry(0);
        header.set_compressed_block_size(compressed_block_size);
        header.set_compression(compression);

        // Convert to little endian
        header.0 = header.0.to_le();
        header
    }

    /// Get the compression preference
    pub fn compression(&self) -> CompressionPreference {
        match self.compression_raw() {
            0 => CompressionPreference::Copy,
            1 => CompressionPreference::ZStandard,
            2 => CompressionPreference::Lz4,
            _ => CompressionPreference::NoPreference,
        }
    }

    /// Set the compression preference
    fn set_compression(&mut self, pref: CompressionPreference) {
        self.set_compression_raw(match pref {
            CompressionPreference::NoPreference => 7, // Using 7 as it's the max value for u3
            CompressionPreference::Copy => 0,
            CompressionPreference::ZStandard => 1,
            CompressionPreference::Lz4 => 2,
        });
    }
}
