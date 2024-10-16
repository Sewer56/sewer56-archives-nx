use core::hint::unreachable_unchecked;

use bitfield::bitfield;

use crate::{api::enums::*, utilities::serialize::little_endian_reader::LittleEndianReader};

bitfield! {
    /// Native 'block entry' in the 'Table of Contents'
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct NativeV1TocBlockEntry(u32);
    impl Debug;
    u32;

    /// `u29` The compressed size of the block.
    pub compressed_block_size, set_compressed_block_size: 31, 3;

    // Keep the raw getter/setter, but make them private
    compression_raw, set_compression_raw: 2, 0;
}

impl NativeV1TocBlockEntry {
    /// Create a new NativeTocBlockEntry
    pub fn new(compressed_block_size: u32, compression: CompressionPreference) -> Self {
        let mut header = NativeV1TocBlockEntry(0);
        header.set_compressed_block_size(compressed_block_size);
        header.set_compression(compression);

        // Convert to little endian
        header.0 = header.0.to_le();
        header
    }

    /// Creates a new entry from the little endian reader
    pub fn from_reader(reader: &mut LittleEndianReader) -> Self {
        NativeV1TocBlockEntry(unsafe { reader.read::<u32>() })
    }

    /// Get the compression preference
    pub fn compression(&self) -> CompressionPreference {
        match self.compression_raw() {
            0 => CompressionPreference::Copy,
            1 => CompressionPreference::ZStandard,
            2 => CompressionPreference::Lz4,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    /// Set the compression preference
    fn set_compression(&mut self, pref: CompressionPreference) {
        self.set_compression_raw(match pref {
            // All cases of 'no preference' should be overwritten with zstd by default.
            CompressionPreference::NoPreference => unsafe { unreachable_unchecked() },
            CompressionPreference::Copy => 0,
            CompressionPreference::ZStandard => 1,
            CompressionPreference::Lz4 => 2,
        });
    }
}
