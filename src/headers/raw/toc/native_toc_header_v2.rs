use crate::headers::enums::v2::*;
use crate::headers::traits::can_convert_to_little_endian::CanConvertToLittleEndian;
use bitfield::bitfield;

bitfield! {
    /// Represents the native structure of the Table of Contents header
    /// for Version 2 of the Table of Contents.
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
    /// It is assumed that [NativeTocHeader] is always little endian.
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NativeTocHeaderV2(u64);
    impl Debug;
    u64;

    /// `u18` The FileCount (18 bits).
    pub file_count, set_file_count: 17, 0;
    /// `u20` The BlockCount (20 bits).
    pub block_count, set_block_count: 37, 18;
    /// `u24`: The size of the (compressed) string pool (24 bits).
    pub string_pool_size, set_string_pool_size: 61, 38;
    /// `u2`` The version (2 bits).
    pub u8, version, set_version: 63, 62;
}

impl NativeTocHeaderV2 {
    pub const SIZE_BYTES: usize = 8;

    /// Initializes the header with given data.
    ///
    /// This is the only way to create and modify a NativeTocHeader.
    /// The returned header is in little-endian format.
    ///
    /// # Arguments
    ///
    /// * `file_count` - The number of files (18 bits).
    /// * `block_count` - The number of blocks (20 bits).
    /// * `string_pool_size` - The size of the string pool (24 bits).
    /// * `version` - The version of the table of contents.
    pub fn init(file_count: u32, block_count: u32, string_pool_size: u32) -> Self {
        let mut header = Self(0);
        header.set_string_pool_size(string_pool_size as u64);
        header.set_block_count(block_count as u64);
        header.set_file_count(file_count as u64);
        header.set_version(2);
        header.to_le()
    }

    /// Creates a NativeTocHeader from a raw u64 value.
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
    /// A new NativeTocHeader instance.
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

    /// Gets the Version as TableOfContentsVersion enum.
    pub fn get_version_unchecked(&self) -> TableOfContentsVersion {
        unsafe { Self::get_version(self).unwrap_unchecked() }
    }
}

impl Default for NativeTocHeaderV2 {
    fn default() -> Self {
        Self::init(0, 0, 0)
    }
}

impl CanConvertToLittleEndian for NativeTocHeaderV2 {
    #[inline(always)]
    fn to_le(&self) -> Self {
        Self(self.0.to_le())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn header_size_is_correct() {
        assert_eq!(
            size_of::<NativeTocHeaderV2>(),
            NativeTocHeaderV2::SIZE_BYTES,
            "NativeTocHeaderV2 size should be {} bytes",
            NativeTocHeaderV2::SIZE_BYTES
        );
    }

    #[test]
    fn can_init_max_values() {
        let header = NativeTocHeaderV2::init(0x3FFFF, 0xFFFFF, 0xFFFFFF);
        assert_eq!(
            header.file_count(),
            0x3FFFF,
            "file_count should be the maximum 18-bit value (0x3FFFF)"
        );
        assert_eq!(
            header.block_count(),
            0xFFFFF,
            "block_count should be the maximum 20-bit value (0xFFFFF)"
        );
        assert_eq!(
            header.string_pool_size(),
            0xFFFFFF,
            "string_pool_size should be the maximum 24-bit value (0xFFFFFF)"
        );
        assert_eq!(
            header.get_version_unchecked(),
            TableOfContentsVersion::V2,
            "Version should be set to V2"
        );
    }

    #[test]
    fn values_correctly_overflow() {
        let header = NativeTocHeaderV2::init(0x40000, 0x100000, 0x1000000);
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
        assert_eq!(
            header.string_pool_size(),
            0,
            "string_pool_size should overflow and wrap to 0"
        );
    }

    #[test]
    fn is_little_endian() {
        let header = NativeTocHeaderV2::init(0x3FFFF, 0xFFFFF, 0xFFFFFF);
        let le_header = header.to_le();
        assert_eq!(
            header.0, le_header.0,
            "Header should be in little-endian format"
        );
    }

    #[test]
    fn default_values_are_sane() {
        let header = NativeTocHeaderV2::default();
        assert_eq!(header.file_count(), 0, "Default file_count should be 0");
        assert_eq!(header.block_count(), 0, "Default block_count should be 0");
        assert_eq!(
            header.string_pool_size(),
            0,
            "Default string_pool_size should be 0"
        );
        assert_eq!(
            header.get_version_unchecked(),
            TableOfContentsVersion::V2,
            "Default version should be V2"
        );
    }
}
