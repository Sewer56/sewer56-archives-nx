use crate::headers::enums::v2::*;
use crate::headers::traits::can_convert_to_little_endian::CanConvertToLittleEndian;
use bitfield::bitfield;

/// Maximum possible size of the string pool for V3.
/// This is constrained by the `StringPoolSize` field (29 bits).
///
/// Maximum `StringPoolSize` = 2^29 - 1 = 536,870,911
pub const MAX_STRING_POOL_SIZE_V3: usize = 536_870_911; // 2^29 - 1

bitfield! {
    /// Represents the native structure of the Table of Contents header
    /// for Version 3 of the Table of Contents.
    ///
    /// This struct is read-only after initialization to ensure consistent endianness.
    /// Use the [Self::init] function to create and initialize a new instance.
    ///
    /// ## Reading from External Source
    ///
    /// When reading from a file from an external source, such as a pre-generated archive file,
    /// use the [to_le](crate::headers::traits::can_convert_to_little_endian::CanConvertToLittleEndian::to_le)
    /// method to ensure correct endianness.
    ///
    /// It is assumed that [NativeTocHeaderV3] is always little endian.
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NativeTocHeaderV3(u64);
    impl Debug;
    u64;

    /// `u17` Padding (bits 16-0).
    pub padding, set_padding: 16, 0;
    /// `u8` The FileCount (bits 24-17).
    pub file_count, set_file_count: 24, 17;
    /// `u8` The BlockCount (bits 32-25).
    pub block_count, set_block_count: 32, 25;
    /// `u29` The size of the (compressed) string pool (bits 61-33).
    pub string_pool_size, set_string_pool_size: 61, 33;
    /// `u2` The version (bits 63-62).
    pub u8, version, set_version: 63, 62;
}

impl NativeTocHeaderV3 {
    /// Maximum possible size of the Native ToC header.
    pub const SIZE_BYTES: usize = 8;

    /// Initializes the header with given data.
    ///
    /// This is the only way to create and modify a NativeTocHeaderV3.
    /// The returned header is in little-endian format.
    ///
    /// # Arguments
    ///
    /// * `file_count` - The number of files (8 bits).
    /// * `block_count` - The number of blocks (8 bits).
    /// * `string_pool_size` - The size of the string pool (29 bits).
    pub fn init(file_count: u32, block_count: u32, string_pool_size: u32) -> Self {
        let mut header = Self(0);
        header.set_string_pool_size(string_pool_size as u64);
        header.set_block_count(block_count as u64);
        header.set_file_count(file_count as u64);
        header.set_version(3);
        header.set_padding(0); // Ensure padding is zero
        header.to_le()
    }

    /// Creates a NativeTocHeaderV3 from a raw u64 value.
    ///
    /// This method assumes that the input value is in the correct format
    /// and does not perform any validation.
    ///
    /// # Arguments
    ///
    /// * `raw` - The raw u64 value representing the header.
    ///
    /// # Returns
    ///
    /// A new NativeTocHeaderV3 instance.
    pub fn from_raw(raw: u64) -> Self {
        Self(raw.to_le())
    }

    /// Converts the header to little-endian format.
    pub fn to_le(&self) -> Self {
        Self(self.0.to_le())
    }

    /// Gets the Version as TableOfContentsVersion enum.
    pub fn get_version(&self) -> Result<TableOfContentsVersion, u8> {
        TableOfContentsVersion::try_from(self.version())
    }

    /// Gets the Version as TableOfContentsVersion enum without checking.
    pub fn get_version_unchecked(&self) -> TableOfContentsVersion {
        unsafe { Self::get_version(self).unwrap_unchecked() }
    }
}

impl CanConvertToLittleEndian for NativeTocHeaderV3 {
    #[inline(always)]
    fn to_le(&self) -> Self {
        Self(self.0.to_le())
    }
}

impl Default for NativeTocHeaderV3 {
    fn default() -> Self {
        Self::init(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn header_size_is_correct() {
        assert_eq!(
            size_of::<NativeTocHeaderV3>(),
            NativeTocHeaderV3::SIZE_BYTES,
            "NativeTocHeaderV3 size should be {} bytes",
            NativeTocHeaderV3::SIZE_BYTES
        );
    }

    #[test]
    fn can_init_max_values() {
        let header = NativeTocHeaderV3::init(0xFF, 0xFF, MAX_STRING_POOL_SIZE_V3 as u32);
        assert_eq!(
            header.file_count(),
            0xFF,
            "file_count should be the maximum 8-bit value (0xFF)"
        );
        assert_eq!(
            header.block_count(),
            0xFF,
            "block_count should be the maximum 8-bit value (0xFF)"
        );
        assert_eq!(
            header.string_pool_size(),
            MAX_STRING_POOL_SIZE_V3 as u64,
            "string_pool_size should be the maximum 29-bit value ({})",
            MAX_STRING_POOL_SIZE_V3
        );
        assert_eq!(
            header.get_version_unchecked(),
            TableOfContentsVersion::V3,
            "Version should be set to V3"
        );
        assert_eq!(header.padding(), 0, "Padding should be initialized to 0");
    }

    #[test]
    fn values_correctly_overflow() {
        // Attempting to set values beyond their bit limits
        // file_count: 8 bits -> max 255, so 256 should overflow to 0
        // block_count: 8 bits -> max 255, so 256 should overflow to 0
        // string_pool_size: 29 bits -> max 536,870,911, so 536,870,912 should overflow to 0

        // Since file_count and block_count are u8, passing 256 as u8 will wrap to 0
        let header = NativeTocHeaderV3::init(256, 256, 0); // All fields set to 0
        assert_eq!(
            header.file_count(),
            0,
            "file_count should overflow and wrap to 0"
        );
        assert_eq!(
            header.block_count(),
            0,
            "block_count should overflow and wrap to 0"
        );
    }

    #[test]
    fn is_little_endian() {
        let header = NativeTocHeaderV3::init(0xFF, 0xFF, MAX_STRING_POOL_SIZE_V3 as u32);
        let le_header = header.to_le();
        assert_eq!(
            header.0, le_header.0,
            "Header should be in little-endian format"
        );
    }

    #[test]
    fn default_values_are_sane() {
        let header = NativeTocHeaderV3::default();
        assert_eq!(header.file_count(), 0, "Default file_count should be 0");
        assert_eq!(header.block_count(), 0, "Default block_count should be 0");
        assert_eq!(
            header.string_pool_size(),
            0,
            "Default string_pool_size should be 0"
        );
        assert_eq!(
            header.get_version_unchecked(),
            TableOfContentsVersion::V3,
            "Default version should be V3"
        );
        assert_eq!(header.padding(), 0, "Default padding should be 0");
    }

    #[test]
    fn from_raw_creates_correct_header() {
        let raw: u64 = 0xC000_0000_0000_0000; // Version = 3 (binary: 11)
        let header = NativeTocHeaderV3::from_raw(raw);
        assert_eq!(
            header.get_version_unchecked(),
            TableOfContentsVersion::V3,
            "Version should be set to V3"
        );
        // All other fields should be 0
        assert_eq!(header.file_count(), 0);
        assert_eq!(header.block_count(), 0);
        assert_eq!(header.string_pool_size(), 0);
        assert_eq!(header.padding(), 0);
    }
}
