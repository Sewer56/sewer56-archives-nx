use bitfield::bitfield;

bitfield! {
    /// Represents the header structure for dictionary compression data.
    ///
    /// Header Layout (8 bytes / 64 bits):
    /// - `u5`: Unused/Reserved
    /// - `u4`: Version (Always 0)
    /// - `u27`: CompressedSize
    /// - `u28`: DecompressedSize
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DictionaryHeader(u64);
    impl Debug;
    u64;

    /// `u5` Reserved bits for future use
    pub u8, reserved, set_reserved: 63, 59;

    /// `u4` Version of the dictionary format (currently always 0)
    pub u8, version, set_version: 58, 55;

    /// `u27` Size of the compressed dictionary data
    pub u32, compressed_size, set_compressed_size: 54, 28;

    /// `u28` Size of the decompressed dictionary data
    pub u32, decompressed_size, set_decompressed_size: 27, 0;
}

impl DictionaryHeader {
    /// Size of header in bytes
    pub const SIZE_BYTES: usize = 8;

    /// Current version of the dictionary format
    pub const CURRENT_VERSION: u8 = 0;

    /// Creates a new DictionaryHeader with the given sizes.
    ///
    /// # Arguments
    ///
    /// * `compressed_size` - Size of the compressed dictionary data
    /// * `decompressed_size` - Size of the decompressed dictionary data
    pub fn new(compressed_size: u32, decompressed_size: u32) -> Self {
        let mut header = DictionaryHeader(0);
        header.set_reserved(0);
        header.set_version(Self::CURRENT_VERSION);
        header.set_compressed_size(compressed_size);
        header.set_decompressed_size(decompressed_size);
        header
    }

    /// Creates a DictionaryHeader from a raw u64 value.
    ///
    /// # Arguments
    ///
    /// * `raw` - The raw u64 value representing the header
    pub fn from_raw(raw: u64) -> Self {
        DictionaryHeader(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_size_is_correct() {
        assert_eq!(size_of::<DictionaryHeader>(), DictionaryHeader::SIZE_BYTES);
    }

    #[test]
    fn can_set_and_get_all_fields() {
        let mut header = DictionaryHeader::new(0, 0);

        // Test setting maximum values for each field
        header.set_reserved(0x1F); // 5 bits
        header.set_version(0xF); // 4 bits
        header.set_compressed_size(0x7FFFFFF); // 27 bits
        header.set_decompressed_size(0xFFFFFFF); // 28 bits

        // Verify fields were set correctly
        assert_eq!(header.reserved(), 0x1F);
        assert_eq!(header.version(), 0xF);
        assert_eq!(header.compressed_size(), 0x7FFFFFF);
        assert_eq!(header.decompressed_size(), 0xFFFFFFF);
    }

    #[test]
    fn values_are_masked_correctly() {
        let mut header = DictionaryHeader::new(0, 0);

        // Test overflow behavior
        header.set_reserved(0xFF); // Greater than 5 bits
        header.set_version(0xFF); // Greater than 4 bits
        header.set_compressed_size(0xFFFFFFF); // Greater than 27 bits
        header.set_decompressed_size(0xFFFFFFFF); // Greater than 28 bits

        // Verify values were masked correctly
        assert_eq!(header.reserved(), 0x1F); // 5 bits
        assert_eq!(header.version(), 0xF); // 4 bits
        assert_eq!(header.compressed_size(), 0x7FFFFFF); // 27 bits
        assert_eq!(header.decompressed_size(), 0xFFFFFFF); // 28 bits
    }

    #[test]
    fn new_sets_correct_defaults() {
        let header = DictionaryHeader::new(123, 456);

        assert_eq!(header.reserved(), 0);
        assert_eq!(header.version(), DictionaryHeader::CURRENT_VERSION);
        assert_eq!(header.compressed_size(), 123);
        assert_eq!(header.decompressed_size(), 456);
    }

    #[test]
    fn from_raw_works() {
        let original = DictionaryHeader::new(123, 456);
        let raw = original.0;
        let from_raw = DictionaryHeader::from_raw(raw);

        assert_eq!(original.0, from_raw.0);
        assert_eq!(from_raw.compressed_size(), 123);
        assert_eq!(from_raw.decompressed_size(), 456);
    }

    #[test]
    fn fields_do_not_overlap() {
        let mut header = DictionaryHeader::new(0, 0);

        // Set all fields to maximum values
        header.set_reserved(0x1F);
        header.set_version(0xF);
        header.set_compressed_size(0x7FFFFFF);
        header.set_decompressed_size(0xFFFFFFF);

        // Set each field to 0 individually and verify others aren't affected
        header.set_reserved(0);
        assert_eq!(header.version(), 0xF);
        assert_eq!(header.compressed_size(), 0x7FFFFFF);
        assert_eq!(header.decompressed_size(), 0xFFFFFFF);

        header.set_version(0);
        assert_eq!(header.reserved(), 0);
        assert_eq!(header.compressed_size(), 0x7FFFFFF);
        assert_eq!(header.decompressed_size(), 0xFFFFFFF);

        header.set_compressed_size(0);
        assert_eq!(header.reserved(), 0);
        assert_eq!(header.version(), 0);
        assert_eq!(header.decompressed_size(), 0xFFFFFFF);

        header.set_decompressed_size(0);
        assert_eq!(header.reserved(), 0);
        assert_eq!(header.version(), 0);
        assert_eq!(header.compressed_size(), 0);
    }
}
