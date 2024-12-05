use bitfield::bitfield;

use super::can_fit_within_42_bits;

bitfield! {
    /// Represents the TOC Header structure.
    ///
    /// The TOC Header is an 8-byte (64-bit) structure with the following bit layout:
    ///
    /// | Bits    | Field                          | Description                                                  |
    /// |---------|--------------------------------|--------------------------------------------------------------|
    /// | 63      | `IsFlexibleFormat`             | Always `1`                                                   |
    /// | 62      | `HasHash`                      | Indicates if a hash is present                               |
    /// | 61 - 57 | `StringPoolSizeBits`           | Number of bits for `StringPoolSize` in `Item Counts`        |
    /// | 56 - 52 | `FileCountBits`                | Number of bits for `FileCount` in `Item Counts`             |
    /// | 51 - 47 | `BlockCountBits`               | Number of bits for `BlockCount` in `Item Counts`            |
    /// | 46 - 42 | `DecompressedBlockOffsetBits`  | Number of bits for `DecompressedBlockOffset` in `Item Counts` |
    /// | 41 - 0  | `PaddingOrItemCounts`          | Padding (aligned to 8 bytes) or `ItemCounts` if it fits      |
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Fef64TocHeader(u64);
    impl Debug;
    u64;

    /// `IsFlexibleFormat` (1 bit) - Bit 63
    pub u8, is_flexible_format, set_is_flexible_format: 63;

    /// `HasHash` (1 bit) - Bit 62
    pub u8, has_hash, set_has_hash: 62;

    /// `StringPoolSizeBits` (5 bits) - Bits 61 to 57
    pub u8, string_pool_size_bits, set_string_pool_size_bits: 61,57;

    /// `FileCountBits` (5 bits) - Bits 56 to 52
    pub u8, file_count_bits, set_file_count_bits: 56,52;

    /// `BlockCountBits` (5 bits) - Bits 51 to 47
    pub u8, block_count_bits, set_block_count_bits: 51,47;

    /// `DecompressedBlockOffsetBits` (5 bits) - Bits 46 to 42
    pub u8, decompressed_block_offset_bits, set_decompressed_block_offset_bits: 46,42;

    /// `PaddingOrItemCounts` (42 bits) - Bits 41 to 0
    pub padding_or_item_counts, set_padding_or_item_counts: 41,0;

}

impl Fef64TocHeader {
    /// Maximum possible size of the TOC Header.
    pub const SIZE_BYTES: usize = 8;

    /// Initializes the TOC Header with given data.
    ///
    /// - Sets `IsFlexibleFormat` to `1` as per specification.
    ///
    /// # Arguments
    ///
    /// * `has_hash` - Indicates if a hash is present (1 bit).
    /// * `string_pool_size_bits` - Number of bits for `StringPoolSize` in `Item Counts` (5 bits).
    /// * `file_count_bits` - Number of bits for `FileCount` in `Item Counts` (5 bits).
    /// * `block_count_bits` - Number of bits for `BlockCount` in `Item Counts` (5 bits).
    /// * `decompressed_block_offset_bits` - Number of bits for `DecompressedBlockOffset` in `Item Counts` (5 bits).
    /// * `padding_or_item_counts` - Padding or `ItemCounts` if it fits (42 bits).
    ///
    /// # Returns
    ///
    /// A new `TocHeader` instance.
    pub fn new(
        has_hash: bool,
        string_pool_size_bits: u8,
        file_count_bits: u8,
        block_count_bits: u8,
        decompressed_block_offset_bits: u8,
        padding_or_item_counts: u64,
    ) -> Self {
        let mut header = Fef64TocHeader(0);
        header.set_is_flexible_format(true);
        header.set_has_hash(has_hash);
        header.set_string_pool_size_bits(string_pool_size_bits);
        header.set_file_count_bits(file_count_bits);
        header.set_block_count_bits(block_count_bits);
        header.set_decompressed_block_offset_bits(decompressed_block_offset_bits);
        header.set_padding_or_item_counts(padding_or_item_counts);
        header
    }

    /// Creates a `TocHeader` from a raw `u64` value.
    ///
    /// # Arguments
    ///
    /// * `raw` - The raw `u64` value representing the header.
    ///
    /// # Returns
    ///
    /// A new `TocHeader` instance.
    pub fn from_raw(raw: u64) -> Self {
        Fef64TocHeader(raw.to_le())
    }

    /// Converts the `TocHeader` to little-endian format.
    pub fn to_le(&self) -> Self {
        Fef64TocHeader(self.0.to_le())
    }

    /// Gets the `IsFlexibleFormat` field.
    pub fn get_is_flexible_format(&self) -> bool {
        self.is_flexible_format()
    }

    /// Gets the `HasHash` field.
    pub fn get_has_hash(&self) -> bool {
        self.has_hash()
    }

    /// Gets the `StringPoolSizeBits` field.
    pub fn get_string_pool_size_bits(&self) -> u8 {
        self.string_pool_size_bits()
    }

    /// Gets the `FileCountBits` field.
    pub fn get_file_count_bits(&self) -> u8 {
        self.file_count_bits()
    }

    /// Gets the `BlockCountBits` field.
    pub fn get_block_count_bits(&self) -> u8 {
        self.block_count_bits()
    }

    /// Gets the `DecompressedBlockOffsetBits` field.
    pub fn get_decompressed_block_offset_bits(&self) -> u8 {
        self.decompressed_block_offset_bits()
    }

    /// Gets the `PaddingOrItemCounts` field.
    pub fn get_padding_or_item_counts(&self) -> u64 {
        self.padding_or_item_counts()
    }

    /// Returns true if the header stores `PaddingOrItemCounts` in an extended
    /// 8 byte header.
    pub fn has_extended_header(&self) -> bool {
        can_fit_within_42_bits(
            self.string_pool_size_bits(),
            self.block_count_bits(),
            self.file_count_bits(),
        )
    }
}

impl Default for Fef64TocHeader {
    fn default() -> Self {
        Self::new(false, 0, 0, 0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn header_size_is_correct() {
        assert_eq!(
            size_of::<Fef64TocHeader>(),
            Fef64TocHeader::SIZE_BYTES,
            "TocHeader size should be 8 bytes"
        );
    }

    #[test]
    fn can_init_max_values() {
        let header = Fef64TocHeader::new(
            true,             // has_hash
            31,               // string_pool_size_bits (5 bits max)
            31,               // file_count_bits (5 bits max)
            31,               // block_count_bits (5 bits max)
            31,               // decompressed_block_offset_bits (5 bits max)
            0x003F_FFFF_FFFF, // padding_or_item_counts (42 bits max)
        );

        assert!(header.get_is_flexible_format());
        assert!(header.get_has_hash());
        assert_eq!(header.get_string_pool_size_bits(), 31);
        assert_eq!(header.get_file_count_bits(), 31);
        assert_eq!(header.get_block_count_bits(), 31);
        assert_eq!(header.get_decompressed_block_offset_bits(), 31);
        assert_eq!(header.get_padding_or_item_counts(), 0x003F_FFFF_FFFF);
    }

    #[test]
    fn values_correctly_overflow() {
        let header = Fef64TocHeader::new(
            true,
            0b1_00000,        // 32 (exceeds 5 bits, should truncate to 0)
            0x20,             // 32 (exceeds 5 bits, should wrap to 0)
            0xFF,             // 255 (exceeds 5 bits, should truncate to 31)
            0x20,             // 32 (exceeds 5 bits, should truncate to 0)
            0xFFFF_FFFF_FFFF, // 48 bits (exceeds 42 bits, should truncate to 0x3FFFFF_FFFF)
        );

        // Fields should truncate to 0 when overflowed.
        assert_eq!(header.get_string_pool_size_bits(), 0);
        assert_eq!(header.get_file_count_bits(), 0);
        assert_eq!(header.get_block_count_bits(), 31);
        assert_eq!(header.get_decompressed_block_offset_bits(), 0);
        assert_eq!(header.get_padding_or_item_counts(), 0x03FF_FFFF_FFFF);
    }

    #[test]
    fn default_values_are_sane() {
        let header = Fef64TocHeader::default();
        assert!(header.get_is_flexible_format());
        assert!(!header.get_has_hash());
        assert_eq!(header.get_string_pool_size_bits(), 0);
        assert_eq!(header.get_file_count_bits(), 0);
        assert_eq!(header.get_block_count_bits(), 0);
        assert_eq!(header.get_decompressed_block_offset_bits(), 0);
        assert_eq!(header.get_padding_or_item_counts(), 0);
    }
}
