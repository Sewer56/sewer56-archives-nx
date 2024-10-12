use bitfield::bitfield;

pub const PRESET3_STRING_POOL_SIZE_MAX: u32 = (1 << 13) - 1; // u13
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
    /// | 59 - 47 | `StringPoolSize`    | Size of the string pool (13 bits)          |
    /// | 46 - 32 | `BlockCount`        | Number of blocks (15 bits)                 |
    /// | 31 - 17 | `FileCount`         | Number of files (15 bits)                  |
    /// | 16 - 0  | `Padding`           | Padding (17 bits)                          |
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Preset3TocHeader(u64);
    impl Debug;

    /// `IsFlexibleFormat` (1 bit) - Bit 63
    pub bool, is_flexible_format, set_is_flexible_format: 63;

    /// `Preset` (2 bits) - Bits 62 to 61
    pub u8, preset, set_preset: 62,61;

    /// `HasHash` (1 bit) - Bit 60
    pub bool, has_hash, set_has_hash: 60;

    /// `StringPoolSize` (13 bits) - Bits 59 to 47
    pub u16, string_pool_size, set_string_pool_size: 59,47;

    /// `BlockCount` (15 bits) - Bits 46 to 32
    pub u16, block_count, set_block_count: 46,32;

    /// `FileCount` (15 bits) - Bits 31 to 17
    pub u16, file_count, set_file_count: 31,17;

    /// `Padding` (17 bits) - Bits 16 to 0
    pub u32, padding, set_padding: 16,0;
}

impl Preset3TocHeader {
    /// Maximum possible size of the Preset3 TOC Header.
    pub const SIZE_BYTES: usize = 8;

    /// Initializes the Preset3 TOC Header with given data.
    ///
    /// - Sets `IsFlexibleFormat` to `0` as per specification.
    /// - Sets `Preset` to `3` as per specification.
    /// - Sets `HasHash` to `true` by default.
    ///
    /// # Arguments
    ///
    /// * `has_hash` - Indicates presence of hash (1 bit).
    /// * `string_pool_size` - Size of the string pool (13 bits).
    /// * `block_count` - Number of blocks (15 bits).
    /// * `file_count` - Number of files (15 bits).
    ///
    /// # Returns
    ///
    /// A new `Preset3TocHeader` instance in little-endian format.
    pub fn new(has_hash: bool, string_pool_size: u16, block_count: u16, file_count: u16) -> Self {
        let mut header = Preset3TocHeader(0);
        header.set_is_flexible_format(false);
        header.set_preset(3);
        header.set_has_hash(has_hash);
        header.set_string_pool_size(string_pool_size);
        header.set_block_count(block_count);
        header.set_file_count(file_count);
        header.set_padding(0); // Initialize padding to 0
        header.to_le()
    }

    /// Creates a `Preset3TocHeader` from a raw `u64` value.
    ///
    /// This method assumes that the input value is in little-endian format
    /// and does not perform any validation.
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
    pub fn get_string_pool_size(&self) -> u16 {
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
    pub fn get_padding(&self) -> u32 {
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
            true, 0x1FFF, // Max 13 bits
            0x7FFF, // Max 15 bits
            0x7FFF, // Max 15 bits
        );

        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 3);
        assert!(header.get_has_hash()); // `HasHash` should default to true
        assert_eq!(header.get_string_pool_size(), 0x1FFF);
        assert_eq!(header.get_block_count(), 0x7FFF);
        assert_eq!(header.get_file_count(), 0x7FFF);
        assert_eq!(header.get_padding(), 0);
    }

    #[test]
    fn values_correctly_overflow() {
        let header = Preset3TocHeader::new(
            true, 0x2000, // 14 bits, should truncate to 13 bits (0)
            0x8000, // 16 bits, should truncate to 15 bits (0)
            0x8000, // 16 bits, should truncate to 15 bits (0)
        );

        assert_eq!(header.get_string_pool_size(), 0);
        assert_eq!(header.get_block_count(), 0);
        assert_eq!(header.get_file_count(), 0);
        assert_eq!(header.get_padding(), 0);
    }

    #[test]
    fn is_little_endian() {
        let header = Preset3TocHeader::new(
            true, 0x1234, // string_pool_size
            0xABCD, // block_count
            0x0123, // file_count
        );
        let le_header = header.to_le();
        assert_eq!(header.0, le_header.0);
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
