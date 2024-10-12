use bitfield::bitfield;

pub const PRESET3_STRING_POOL_SIZE_MAX: u32 = (1 << 20) - 1; // u20
pub const PRESET3_BLOCK_COUNT_MAX: u32 = (1 << 16) - 1; // u16
pub const PRESET3_FILE_COUNT_MAX: u32 = (1 << 16) - 1; // u16
pub const PRESET3_MAX_DECOMPRESSED_BLOCK_OFFSET: u32 = 0; // Not used in Preset3
pub const PRESET3_MAX_FILE_SIZE: u32 = ((1_u64 << 32) - 1) as u32; // 4 GiB

bitfield! {
    /// Represents the TOC Header structure for Preset 3.
    ///
    /// The TOC Header is an 8-byte (64-bit) structure with the following bit layout:
    ///
    /// | Bits    | Field               | Description                                |
    /// |---------|---------------------|--------------------------------------------|
    /// | 63      | `IsFlexibleFormat`  | Always `0`                                 |
    /// | 62 - 61 | `Preset`            | Always `3`                                 |
    /// | 60      | `HasHash`           | Indicates presence of hash (1 bit)         |
    /// | 59 - 40 | `StringPoolSize`    | Size of the string pool (20 bits)          |
    /// | 39 - 24 | `BlockCount`        | Number of blocks (16 bits)                 |
    /// | 23 - 8  | `FileCount`         | Number of files (16 bits)                  |
    /// | 7 - 0   | `Padding`           | Padding (8 bits)                           |
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Preset3TocHeader(u64);
    impl Debug;

    /// `IsFlexibleFormat` (1 bit) - Bit 63
    pub bool, is_flexible_format, set_is_flexible_format: 63;

    /// `Preset` (2 bits) - Bits 62 to 61
    pub u8, preset, set_preset: 62,61;

    /// `HasHash` (1 bit) - Bit 60
    pub bool, has_hash, set_has_hash: 60;

    /// `StringPoolSize` (20 bits) - Bits 59 to 40
    pub u32, string_pool_size, set_string_pool_size: 59,40;

    /// `BlockCount` (16 bits) - Bits 39 to 24
    pub u16, block_count, set_block_count: 39,24;

    /// `FileCount` (16 bits) - Bits 23 to 8
    pub u16, file_count, set_file_count: 23,8;

    /// `Padding` (8 bits) - Bits 7 to 0
    pub u8, padding, set_padding: 7,0;
}

impl Preset3TocHeader {
    /// Maximum possible size of the Preset3 TOC Header.
    pub const SIZE_BYTES: usize = 8;

    /// Initializes the Preset3 TOC Header with given data.
    ///
    /// - Sets `IsFlexibleFormat` to `0` as per specification.
    /// - Sets `Preset` to `3` as per specification.
    ///
    /// # Arguments
    ///
    /// * `has_hash` - Indicates presence of hash (1 bit).
    /// * `string_pool_size` - Size of the string pool (20 bits).
    /// * `block_count` - Number of blocks (16 bits).
    /// * `file_count` - Number of files (16 bits).
    ///
    /// # Returns
    ///
    /// A new `Preset3TocHeader` instance.
    pub fn new(has_hash: bool, string_pool_size: u32, block_count: u16, file_count: u16) -> Self {
        let mut header = Preset3TocHeader(0);
        header.set_is_flexible_format(false);
        header.set_preset(3);
        header.set_has_hash(has_hash);
        header.set_string_pool_size(string_pool_size);
        header.set_block_count(block_count);
        header.set_file_count(file_count);
        header.set_padding(0); // Initialize padding to 0
        header
    }

    /// Creates a `Preset3TocHeader` from a raw `u64` value.
    /// This method does not perform any validation.
    ///
    /// # Arguments
    ///
    /// * `raw` - The raw `u64` value representing the header.
    ///
    /// # Returns
    ///
    /// A new `Preset3TocHeader` instance.
    #[coverage(off)]
    pub fn from_raw(raw: u64) -> Self {
        Preset3TocHeader(raw.to_le())
    }

    /// Converts the `Preset3TocHeader` to little-endian format.
    pub fn to_le(&self) -> Self {
        Preset3TocHeader(self.0.to_le())
    }

    /// Gets the `IsFlexibleFormat` field.
    pub fn get_is_flexible_format(&self) -> bool {
        self.is_flexible_format()
    }

    /// Gets the `Preset` field.
    pub fn get_preset(&self) -> u8 {
        self.preset()
    }

    /// Gets the `HasHash` field.
    pub fn get_has_hash(&self) -> bool {
        self.has_hash()
    }

    /// Gets the `StringPoolSize` field.
    pub fn get_string_pool_size(&self) -> u32 {
        self.string_pool_size()
    }

    /// Gets the `BlockCount` field.
    pub fn get_block_count(&self) -> u16 {
        self.block_count()
    }

    /// Gets the `FileCount` field.
    pub fn get_file_count(&self) -> u16 {
        self.file_count()
    }

    /// Gets the `Padding` field.
    pub fn get_padding(&self) -> u8 {
        self.padding()
    }
}

impl Default for Preset3TocHeader {
    fn default() -> Self {
        Self::new(true, 0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn header_size_is_correct() {
        assert_eq!(size_of::<Preset3TocHeader>(), Preset3TocHeader::SIZE_BYTES);
    }

    #[test]
    fn can_init_max_values() {
        let header = Preset3TocHeader::new(
            true,
            PRESET3_STRING_POOL_SIZE_MAX,   // Max 20 bits
            PRESET3_BLOCK_COUNT_MAX as u16, // Max 16 bits
            PRESET3_FILE_COUNT_MAX as u16,  // Max 16 bits
        );

        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 3);
        assert!(header.get_has_hash());
        assert_eq!(header.get_string_pool_size(), PRESET3_STRING_POOL_SIZE_MAX);
        assert_eq!(header.get_block_count(), PRESET3_BLOCK_COUNT_MAX as u16);
        assert_eq!(header.get_file_count(), PRESET3_FILE_COUNT_MAX as u16);
        assert_eq!(header.get_padding(), 0);
    }

    #[test]
    fn values_correctly_overflow() {
        let header = Preset3TocHeader::new(
            true,
            PRESET3_STRING_POOL_SIZE_MAX + 1, // 21 bits, should truncate to 20 bits
            (PRESET3_BLOCK_COUNT_MAX + 1) as u16, // 17 bits, should truncate to 16 bits
            (PRESET3_FILE_COUNT_MAX + 1) as u16, // 17 bits, should truncate to 16 bits
        );

        assert_eq!(header.get_string_pool_size(), 0);
        assert_eq!(header.get_block_count(), 0);
        assert_eq!(header.get_file_count(), 0);
        assert_eq!(header.get_padding(), 0);
    }

    #[test]
    fn default_values_are_sane() {
        let header = Preset3TocHeader::default();
        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 3);
        assert!(header.get_has_hash()); // `HasHash` should default to true
        assert_eq!(header.get_string_pool_size(), 0);
        assert_eq!(header.get_block_count(), 0);
        assert_eq!(header.get_file_count(), 0);
        assert_eq!(header.get_padding(), 0);
    }
}
