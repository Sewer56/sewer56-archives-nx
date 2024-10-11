use bitfield::bitfield;

bitfield! {
    /// Represents the TOC Header structure for Preset 3.
    ///
    /// The TOC Header is an 8-byte (64-bit) structure with the following bit layout:
    ///
    /// | Bits    | Field               | Description                                |
    /// |---------|---------------------|--------------------------------------------|
    /// | 63      | `IsFlexibleFormat`  | Always `0`                                 |
    /// | 62 - 61 | `Preset`            | Always `3`                                 |
    /// | 60 - 48 | `StringPoolSize`    | Size of the string pool (13 bits)          |
    /// | 47 - 33 | `BlockCount`        | Number of blocks (15 bits)                 |
    /// | 32 - 18 | `FileCount`         | Number of files (15 bits)                  |
    /// | 17 - 0  | `Padding`           | Padding (18 bits)                          |
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Preset3TocHeader(u64);
    impl Debug;

    /// `IsFlexibleFormat` (1 bit) - Bit 63
    pub bool, is_flexible_format, set_is_flexible_format: 63;

    /// `Preset` (2 bits) - Bits 62 to 61
    pub u8, preset, set_preset: 62,61;

    /// `StringPoolSize` (13 bits) - Bits 60 to 48
    pub u16, string_pool_size, set_string_pool_size: 60,48;

    /// `BlockCount` (15 bits) - Bits 47 to 33
    pub u16, block_count, set_block_count: 47,33;

    /// `FileCount` (15 bits) - Bits 32 to 18
    pub u16, file_count, set_file_count: 32,18;

    /// `Padding` (18 bits) - Bits 17 to 0
    pub u32, padding, set_padding: 17,0;
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
    /// * `string_pool_size` - Size of the string pool (13 bits).
    /// * `block_count` - Number of blocks (15 bits).
    /// * `file_count` - Number of files (15 bits).
    ///
    /// # Returns
    ///
    /// A new `Preset3TocHeader` instance in little-endian format.
    pub fn new(string_pool_size: u16, block_count: u16, file_count: u16) -> Self {
        let mut header = Preset3TocHeader(0);
        header.set_is_flexible_format(false);
        header.set_preset(3);
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
        Self::new(0, 0, 0)
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
            0x1FFF, // Max 13 bits
            0x7FFF, // Max 15 bits
            0x7FFF, // Max 15 bits
        );

        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 3);
        assert_eq!(header.get_string_pool_size(), 0x1FFF);
        assert_eq!(header.get_block_count(), 0x7FFF);
        assert_eq!(header.get_file_count(), 0x7FFF);
        assert_eq!(header.get_padding(), 0);
    }

    #[test]
    fn values_correctly_overflow() {
        let header = Preset3TocHeader::new(
            0x2000, // 14 bits, should truncate to 13 bits (0)
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
            0x1234, // string_pool_size
            0xABCD, // block_count
            0x0123, // file_count (Note: 0x2ABCDE exceeds 15 bits and will be truncated)
        );
        let le_header = header.to_le();
        assert_eq!(header.0, le_header.0);
    }

    #[test]
    fn default_values_are_sane() {
        let header = Preset3TocHeader::default();
        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 3);
        assert_eq!(header.get_string_pool_size(), 0);
        assert_eq!(header.get_block_count(), 0);
        assert_eq!(header.get_file_count(), 0);
        assert_eq!(header.get_padding(), 0);
    }

    #[test]
    fn from_raw_creates_correct_header() {
        // Set bits62-61 to '11' for Preset 3
        let raw: u64 = 0x60123456789ABCDE;
        let header = Preset3TocHeader::from_raw(raw);

        assert!(!header.get_is_flexible_format());
        assert_eq!(header.get_preset(), 3);
        assert_eq!(header.get_string_pool_size(), ((raw >> 48) & 0x1FFF) as u16);
        assert_eq!(header.get_block_count(), ((raw >> 33) & 0x7FFF) as u16);
        assert_eq!(header.get_file_count(), ((raw >> 18) & 0x7FFF) as u16);
        assert_eq!(header.get_padding(), (raw & 0x3FFFF) as u32);
    }
}
