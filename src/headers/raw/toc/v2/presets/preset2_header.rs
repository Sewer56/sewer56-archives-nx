use bitfield::bitfield;

pub const PRESET2_STRING_POOL_SIZE_MAX: u32 = (1 << 21) - 1; // u21
pub const PRESET2_BLOCK_COUNT_MAX: u32 = (1 << 22) - 1; // u22
pub const PRESET2_FILE_COUNT_MAX: u32 = (1 << 18) - 1; // u18
pub const PRESET2_MAX_FILE_SIZE: u64 = u64::MAX;

bitfield! {
    /// Represents the TOC Header structure for Preset 2.
    ///
    /// The TOC Header is an 8-byte (64-bit) structure with the following bit layout:
    ///
    /// | Bits    | Field               | Description                                |
    /// |---------|---------------------|--------------------------------------------|
    /// | 63      | `IsFlexibleFormat`  | Always `0`                                 |
    /// | 62 - 61 | `Preset`            | Always `2`                                 |
    /// | 60 - 40 | `StringPoolSize`    | Size of the string pool (21 bits)          |
    /// | 39 - 18 | `BlockCount`        | Number of blocks (22 bits)                 |
    /// | 17 - 0  | `FileCount`         | Number of files (18 bits)                  |
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Preset2TocHeader(u64);
    impl Debug;

    /// `IsFlexibleFormat` (1 bit) - Bit 63
    pub bool, is_flexible_format, set_is_flexible_format: 63;

    /// `Preset` (2 bits) - Bits 62 to 61
    pub u8, preset, set_preset: 62,61;

    /// `StringPoolSize` (21 bits) - Bits 60 to 40
    pub u32, string_pool_size, set_string_pool_size: 60,40;

    /// `BlockCount` (22 bits) - Bits 39 to 18
    pub u32, block_count, set_block_count: 39,18;

    /// `FileCount` (18 bits) - Bits 17 to 0
    pub u32, file_count, set_file_count: 17,0;
}

impl Preset2TocHeader {
    /// Maximum possible size of the Preset2 TOC Header.
    pub const SIZE_BYTES: usize = 8;

    /// Initializes the Preset2 TOC Header with given data.
    ///
    /// - Sets `IsFlexibleFormat` to `0` as per specification.
    /// - Sets `Preset` to `2` as per specification.
    ///
    /// # Arguments
    ///
    /// * `string_pool_size` - Size of the string pool (21 bits).
    /// * `block_count` - Number of blocks (22 bits).
    /// * `file_count` - Number of files (18 bits).
    ///
    /// # Returns
    ///
    /// A new `Preset2TocHeader` instance in little-endian format.
    pub fn new(string_pool_size: u32, block_count: u32, file_count: u32) -> Self {
        let mut header = Preset2TocHeader(0);
        header.set_is_flexible_format(false);
        header.set_preset(2);
        header.set_string_pool_size(string_pool_size);
        header.set_block_count(block_count);
        header.set_file_count(file_count);
        header.to_le()
    }

    /// Creates a `Preset2TocHeader` from a raw `u64` value.
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
    /// A new `Preset2TocHeader` instance.
    pub fn from_raw(raw: u64) -> Self {
        Preset2TocHeader(raw.to_le())
    }

    /// Converts the `Preset2TocHeader` to little-endian format.
    pub fn to_le(&self) -> Self {
        Preset2TocHeader(self.0.to_le())
    }

    /// Gets the `IsFlexibleFormat` field.
    pub fn get_is_flexible_format(&self) -> bool {
        self.is_flexible_format()
    }

    /// Gets the `Preset` field.
    pub fn get_preset(&self) -> u8 {
        self.preset()
    }

    /// Gets the `StringPoolSize` field.
    pub fn get_string_pool_size(&self) -> u32 {
        self.string_pool_size()
    }

    /// Gets the `BlockCount` field.
    pub fn get_block_count(&self) -> u32 {
        self.block_count()
    }

    /// Gets the `FileCount` field.
    pub fn get_file_count(&self) -> u32 {
        self.file_count()
    }
}

impl Default for Preset2TocHeader {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn header_size_is_correct() {
        assert_eq!(size_of::<Preset2TocHeader>(), Preset2TocHeader::SIZE_BYTES);
    }

    #[test]
    fn can_init_max_values() {
        let header = Preset2TocHeader::new(
            0x1FFFFF, // Max 21 bits
            0x3FFFFF, // Max 22 bits
            0x3FFFF,  // Max 18 bits
        );

        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 2);
        assert_eq!(header.get_string_pool_size(), 0x1FFFFF);
        assert_eq!(header.get_block_count(), 0x3FFFFF);
        assert_eq!(header.get_file_count(), 0x3FFFF);
    }

    #[test]
    fn values_correctly_overflow() {
        let header = Preset2TocHeader::new(
            0x2_00000,  // 22 bits, should truncate to 21 bits (0)
            0x4_000000, // 24 bits, should truncate to 22 bits (0)
            0x40000,    // 19 bits, should truncate to 18 bits (0)
        );

        assert_eq!(header.get_string_pool_size(), 0);
        assert_eq!(header.get_block_count(), 0);
        assert_eq!(header.get_file_count(), 0);
    }

    #[test]
    fn is_little_endian() {
        let header = Preset2TocHeader::new(
            0x123456, // string_pool_size
            0x1ABCDE, // block_count
            0x2ABCDE, // file_count
        );
        let le_header = header.to_le();
        assert_eq!(header.0, le_header.0);
    }

    #[test]
    fn default_values_are_sane() {
        let header = Preset2TocHeader::default();
        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 2);
        assert_eq!(header.get_string_pool_size(), 0);
        assert_eq!(header.get_block_count(), 0);
        assert_eq!(header.get_file_count(), 0);
    }

    #[test]
    fn from_raw_creates_correct_header() {
        // Set bits62-61 to '10' for Preset 2
        let raw: u64 = 0x40123456789ABCDE;
        let header = Preset2TocHeader::from_raw(raw);

        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 2);
        assert_eq!(
            header.get_string_pool_size(),
            ((raw >> 40) & 0x1FFFFF) as u32
        );
        assert_eq!(header.get_block_count(), ((raw >> 18) & 0x3FFFFF) as u32);
        assert_eq!(header.get_file_count(), (raw & 0x3FFFF) as u32);
    }
}
