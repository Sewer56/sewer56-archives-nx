use crate::headers::enums::table_of_contents_version::TableOfContentsVersion;
use crate::headers::traits::can_convert_to_little_endian::CanConvertToLittleEndian;
use bitfield::bitfield;

bitfield! {
    /// Represents the native structure of the Table of Contents header.
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
    pub struct NativeTocHeader(u64);
    impl Debug;
    u32;

    /// `u20` The FileCount (20 bits).
    pub file_count, set_file_count: 19, 0;
    /// `u18` The BlockCount (18 bits).
    pub block_count, set_block_count: 37, 20;
    /// `u24` The size of the string pool (24 bits).
    pub string_pool_size, set_string_pool_size: 61, 38;
    /// `u2`` The version (2 bits).
    pub u8, version, set_version: 63, 62;
}

impl NativeTocHeader {
    /// Maximum possible size of the string pool.
    /// This is not a format limit, this size is derived from the following approximation:
    ///
    /// - FileEntries = 4.3MiB
    /// - Blocks = 1MiB
    /// - StringPool = 0.66MiB
    ///
    /// StringPool is ~11% of the total size.
    /// If we extrapolate this, and do [256MiB * 0.11 MiB] = 28.16MiB
    /// Where 256MiB is max Nx header size.
    ///
    /// If the string pool is larger than this, the archive will most likely have an insufficient
    /// space in the header. Hence the value is set to this arbitrary limit. On the good news,
    /// to hit this limit you need over 7 million files, which the format doesn't even currently support.
    ///
    /// Data Source:
    ///
    /// Sewer's sample of SteamApps folder with 150+ games and 180k files.
    pub const MAX_STRING_POOL_SIZE: usize = 29360127; // 2^24 - 1

    /// Maximum possible size of the Native ToC header.
    pub const SIZE_BYTES: usize = 8;

    /// Initializes the header with given data.
    ///
    /// This is the only way to create and modify a NativeTocHeader.
    /// The returned header is in little-endian format.
    ///
    /// # Arguments
    ///
    /// * `file_count` - The number of files (20 bits).
    /// * `block_count` - The number of blocks (18 bits).
    /// * `string_pool_size` - The size of the string pool (24 bits).
    /// * `version` - The version of the table of contents.
    pub fn init(
        file_count: u32,
        block_count: u32,
        string_pool_size: u32,
        version: TableOfContentsVersion,
    ) -> Self {
        let mut header = Self(0);
        header.set_file_count(file_count);
        header.set_block_count(block_count);
        header.set_string_pool_size(string_pool_size);
        header.set_version(version as u8);
        header.to_le()
    }

    /// Gets the Version as TableOfContentsVersion enum.
    pub fn get_version(&self) -> TableOfContentsVersion {
        unsafe { TableOfContentsVersion::try_from(self.version()).unwrap_unchecked() }
    }
}

impl CanConvertToLittleEndian for NativeTocHeader {
    fn to_le(&self) -> Self {
        Self(self.0.to_le())
    }
}

impl Default for NativeTocHeader {
    fn default() -> Self {
        Self::init(0, 0, 0, TableOfContentsVersion::V0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_size_is_correct() {
        assert_eq!(size_of::<NativeTocHeader>(), NativeTocHeader::SIZE_BYTES);
    }

    #[test]
    fn can_init_max_values() {
        let header = NativeTocHeader::init(0xFFFFF, 0x3FFFF, 0xFFFFFF, TableOfContentsVersion::V1);
        assert_eq!(header.file_count(), 0xFFFFF);
        assert_eq!(header.block_count(), 0x3FFFF);
        assert_eq!(header.string_pool_size(), 0xFFFFFF);
        assert_eq!(header.get_version(), TableOfContentsVersion::V1);
    }

    #[test]
    fn values_correctly_overflow() {
        let header =
            NativeTocHeader::init(0x100000, 0x40000, 0x1000000, TableOfContentsVersion::V0);
        assert_eq!(header.file_count(), 0);
        assert_eq!(header.block_count(), 0);
        assert_eq!(header.string_pool_size(), 0);
    }

    #[test]
    fn is_little_endian() {
        let header = NativeTocHeader::init(0xFFFFF, 0x3FFFF, 0xFFFFFF, TableOfContentsVersion::V1);
        let le_header = header.to_le();
        assert_eq!(header.0, le_header.0);
    }

    #[test]
    fn default_vales_are_sane() {
        let header = NativeTocHeader::default();
        assert_eq!(header.file_count(), 0);
        assert_eq!(header.block_count(), 0);
        assert_eq!(header.string_pool_size(), 0);
        assert_eq!(header.get_version(), TableOfContentsVersion::V0);
    }
}
