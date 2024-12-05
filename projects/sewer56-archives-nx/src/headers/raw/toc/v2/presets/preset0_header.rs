use bitfield::bitfield;

pub const PRESET0_STRING_POOL_SIZE_MAX: u32 = (1 << 21) - 1; // u21
pub const PRESET0_BLOCK_COUNT_MAX: u32 = (1 << 22) - 1; // u22
pub const PRESET0_FILE_COUNT_MAX: u32 = (1 << 18) - 1; // u18
pub const PRESET0_DECOMPRESSED_BLOCK_OFFSET_MAX: u32 = (1 << 24) - 1; // u24
pub const PRESET0_MAX_FILE_SIZE: u32 = ((1_u64 << 32) - 1) as u32; // 4 GiB

pub const PRESET1_STRING_POOL_SIZE_MAX: u32 = (1 << 21) - 1; // u21
pub const PRESET1_BLOCK_COUNT_MAX: u32 = (1 << 22) - 1; // u22
pub const PRESET1_FILE_COUNT_MAX: u32 = (1 << 18) - 1; // u18
pub const PRESET1_MAX_FILE_SIZE: u32 = ((1_u64 << 32) - 1) as u32; // 4 GiB

pub const PRESET2_STRING_POOL_SIZE_MAX: u32 = (1 << 21) - 1; // u21
pub const PRESET2_BLOCK_COUNT_MAX: u32 = (1 << 22) - 1; // u22
pub const PRESET2_FILE_COUNT_MAX: u32 = (1 << 18) - 1; // u18
pub const PRESET2_MAX_FILE_SIZE: u64 = u64::MAX;

bitfield! {

    /// Represents the TOC Header structure for Preset 0.
    ///
    /// The TOC Header is an 8-byte (64-bit) structure with the following bit layout:
    ///
    /// | Bits    | Field               | Description                                |
    /// |---------|---------------------|--------------------------------------------|
    /// | 63      | `IsFlexibleFormat`  | Always `0`                                 |
    /// | 62 - 61 | `Preset`            | Always `0`                                 |
    /// | 60 - 40 | `StringPoolSize`    | Size of the string pool (21 bits)          |
    /// | 39 - 18 | `BlockCount`        | Number of blocks (22 bits)                 |
    /// | 17 - 0  | `FileCount`         | Number of files (18 bits)                  |
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Preset0TocHeader(u64);
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

impl Preset0TocHeader {
    /// Maximum possible size of the Preset0 TOC Header.
    pub const SIZE_BYTES: usize = 8;

    /// Initializes the Preset0 TOC Header with given data.
    ///
    /// - Sets `IsFlexibleFormat` to `0` as per specification.
    /// - Sets `Preset` to `0` as per specification.
    ///
    /// # Arguments
    ///
    /// * `preset` - The specific preset (can be 0, 1 or 2).
    /// * `string_pool_size` - Size of the string pool (21 bits).
    /// * `block_count` - Number of blocks (22 bits).
    /// * `file_count` - Number of files (18 bits).
    ///
    /// # Returns
    ///
    /// A new `Preset0TocHeader` instance.
    pub fn new(preset: u8, string_pool_size: u32, block_count: u32, file_count: u32) -> Self {
        let mut header = Preset0TocHeader(0);
        header.set_is_flexible_format(false);
        header.set_preset(preset);
        header.set_string_pool_size(string_pool_size);
        header.set_block_count(block_count);
        header.set_file_count(file_count);
        header
    }

    /// Creates a `Preset0TocHeader` from a raw `u64` value.
    /// This method does not perform any validation.
    ///
    /// # Arguments
    ///
    /// * `raw` - The raw `u64` value representing the header.
    ///
    /// # Returns
    ///
    /// A new `Preset0TocHeader` instance.
    pub fn from_raw(raw: u64) -> Self {
        Preset0TocHeader(raw.to_le())
    }

    /// Converts the `Preset0TocHeader` to little-endian format.
    pub fn to_le(&self) -> Self {
        Preset0TocHeader(self.0.to_le())
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

impl Default for Preset0TocHeader {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn header_size_is_correct() {
        assert_eq!(size_of::<Preset0TocHeader>(), Preset0TocHeader::SIZE_BYTES);
    }

    #[test]
    fn can_init_max_values() {
        let header = Preset0TocHeader::new(
            0, 0x1FFFFF, // Max 21 bits
            0x3FFFFF, // Max 22 bits
            0x3FFFF,  // Max 18 bits
        );

        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 0);
        assert_eq!(header.get_string_pool_size(), 0x1FFFFF);
        assert_eq!(header.get_block_count(), 0x3FFFFF);
        assert_eq!(header.get_file_count(), 0x3FFFF);
    }

    #[test]
    fn values_correctly_overflow() {
        let header = Preset0TocHeader::new(
            0, 0x2_00000,  // 22 bits, should truncate to 21 bits (0)
            0x4_000000, // 24 bits, should truncate to 22 bits (0)
            0x40000,    // 19 bits, should truncate to 18 bits (0)
        );

        assert_eq!(header.get_string_pool_size(), 0);
        assert_eq!(header.get_block_count(), 0);
        assert_eq!(header.get_file_count(), 0);
    }

    #[test]
    fn default_values_are_sane() {
        let header = Preset0TocHeader::default();
        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 0);
        assert_eq!(header.get_string_pool_size(), 0);
        assert_eq!(header.get_block_count(), 0);
        assert_eq!(header.get_file_count(), 0);
    }

    #[test]
    #[cfg_attr(target_endian = "big", ignore = "currently fails on big endian")]
    fn from_raw_creates_correct_header() {
        let raw: u64 = 0x000123456789ABCDE;
        let header = Preset0TocHeader::from_raw(raw);

        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 0);
        assert_eq!(
            header.get_string_pool_size(),
            ((raw >> 40) & 0x1FFFFF) as u32
        );
        assert_eq!(header.get_block_count(), ((raw >> 18) & 0x3FFFFF) as u32);
        assert_eq!(header.get_file_count(), (raw & 0x3FFFF) as u32);
    }
}
