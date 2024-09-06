use crate::headers::traits::can_convert_to_little_endian::CanConvertToLittleEndian;
use bitfield::bitfield;

bitfield! {
    /// Packed header data
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct HeaderData(u32);
    impl Debug;
    u32;

    /// `u7` The archive version.
    pub version, set_version: 31, 25;
    /// `u5` The chunk size in its encoded raw value.
    pub chunk_size, set_chunk_size: 24, 20;
    /// `u16` The number of 4K pages used to store the entire header (incl. compressed TOC and stringpool).
    pub header_page_count, set_header_page_count: 19, 4;
    /// `u4` The 'feature flags' for this structure.
    pub feature_flags, set_feature_flags: 3, 0;
}

/// Structure that represents the native serialized file header.
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
/// It is assumed that [NativeFileHeader] is always little endian.
#[repr(C, packed(4))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct NativeFileHeader {
    /// [u32] 'Magic' header to identify the file.
    magic: u32,

    /// Packed values at end of the header.
    header_data: HeaderData,
}

impl NativeFileHeader {
    /// Size of header in bytes.
    pub const SIZE_BYTES: usize = 8;

    /// The current version of the Nexus Archive Format.
    /// TODO: Bump this when new features are added.
    pub const CURRENT_ARCHIVE_VERSION: u8 = 0;

    /// Minimum size of chunk blocks in the Nx archive.
    pub const BASE_CHUNK_SIZE: u32 = 512;

    /// Expected magic number for the NX archive format ('NXUS').
    const EXPECTED_MAGIC: u32 = 0x4E585553_u32.to_le();

    /// Size of a header page in bytes.
    const HEADER_PAGE_SIZE: u32 = 4096;

    /// Returns true if the 'Magic' in the header is valid, else false.
    pub fn is_valid_magic_header(&self) -> bool {
        self.magic == Self::EXPECTED_MAGIC
    }

    /// Gets the total amount of bytes required to fetch this header and the table of contents.
    pub fn header_page_bytes(&self) -> u32 {
        self.header_data.header_page_count() * Self::HEADER_PAGE_SIZE
    }

    /// Gets the chunk size used to split large files by.
    pub fn chunk_size_bytes(&self) -> u32 {
        Self::BASE_CHUNK_SIZE << self.header_data.chunk_size()
    }

    /// Initializes the header with given data.
    ///
    /// This is the only way to create and modify a NativeFileHeader.
    /// The returned header is in little-endian format.
    ///
    /// # Arguments
    ///
    /// * `chunk_size_bytes` - Size of single chunk in archive.
    /// * `header_page_count_bytes` - Number of 4K pages used to store the entire header (incl. compressed TOC and stringpool).
    pub fn init(chunk_size_bytes: u32, header_page_count_bytes: u32) -> Self {
        let mut header = Self {
            magic: Self::EXPECTED_MAGIC,
            header_data: HeaderData(0),
        };

        header
            .header_data
            .set_version(Self::CURRENT_ARCHIVE_VERSION as u32);
        header.set_chunk_size_bytes(
            chunk_size_bytes
                .max(Self::BASE_CHUNK_SIZE)
                .next_power_of_two(),
        );
        header.set_header_page_bytes(
            header_page_count_bytes.next_multiple_of(Self::HEADER_PAGE_SIZE),
        );
        header.to_le()
    }

    /// Sets the chunk size used to split large files by.
    ///
    /// This method calculates the appropriate chunk size value to store in the header.
    /// It assumes the input value is already a power of 2 multiple of BASE_CHUNK_SIZE.
    ///
    /// # Arguments
    /// * `value` - Chunk size in bytes. Must be a power of 2 and at least BASE_CHUNK_SIZE.
    ///
    /// # Panics
    /// Panics if the input value is not a power of 2 or is less than BASE_CHUNK_SIZE.
    fn set_chunk_size_bytes(&mut self, value: u32) {
        debug_assert!(value.is_power_of_two(), "Chunk size must be a power of 2");
        debug_assert!(
            value >= Self::BASE_CHUNK_SIZE,
            "Chunk size must be at least BASE_CHUNK_SIZE"
        );

        // Calculate how many times we need to multiply BASE_CHUNK_SIZE by 2 to get value
        let power = (value / Self::BASE_CHUNK_SIZE).trailing_zeros();

        // Store the calculated power
        self.header_data.set_chunk_size(power);
    }

    /// Sets the total amount of bytes required to fetch this header and the table of contents.
    fn set_header_page_bytes(&mut self, value: u32) {
        self.header_data
            .set_header_page_count(value / Self::HEADER_PAGE_SIZE);
    }
}

impl CanConvertToLittleEndian for NativeFileHeader {
    fn to_le(&self) -> Self {
        Self {
            magic: self.magic.to_le(),
            header_data: self.header_data.to_le(),
        }
    }
}

impl CanConvertToLittleEndian for HeaderData {
    fn to_le(&self) -> Self {
        HeaderData(self.0.to_le())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_file_header_size() {
        assert_eq!(size_of::<NativeFileHeader>(), NativeFileHeader::SIZE_BYTES);
    }

    #[test]
    fn test_native_file_header_init() {
        let header = NativeFileHeader::init(1024, 8192);
        assert!(header.is_valid_magic_header());
        assert_eq!(
            header.header_data.version(),
            NativeFileHeader::CURRENT_ARCHIVE_VERSION as u32
        );
        assert_eq!(header.chunk_size_bytes(), 1024);
        assert_eq!(header.header_page_bytes(), 8192);
    }

    #[test]
    fn test_feature_flags() {
        let header = NativeFileHeader::init(1024, 8192);
        // Note: We can't modify feature_flags directly at the moment, so this test is just checking the initial value
        assert_eq!(header.header_data.feature_flags(), 0);
    }

    #[test]
    fn test_endianness() {
        let header = NativeFileHeader::init(1024, 8192);
        let le_header = header.to_le();
        assert_eq!(header.magic, le_header.magic);
        assert_eq!(header.header_data.0, le_header.header_data.0);
    }

    #[test]
    fn test_header_page_count() {
        let header = NativeFileHeader::init(1024, 8192);
        assert_eq!(header.header_data.header_page_count(), 2);

        let header = NativeFileHeader::init(1024, 16384);
        assert_eq!(header.header_data.header_page_count(), 4);

        let header = NativeFileHeader::init(1024, 20480); // Not a multiple of 4096
        assert_eq!(header.header_data.header_page_count(), 5); // Should round up to 5 pages (20480 bytes)
    }

    #[test]
    fn test_chunk_size() {
        // Test with exact powers of 2 multiples of BASE_CHUNK_SIZE
        let header = NativeFileHeader::init(512, 8192);
        assert_eq!(header.header_data.chunk_size(), 0);
        assert_eq!(header.chunk_size_bytes(), 512);

        let header = NativeFileHeader::init(1024, 8192);
        assert_eq!(header.header_data.chunk_size(), 1);
        assert_eq!(header.chunk_size_bytes(), 1024);

        let header = NativeFileHeader::init(2048, 8192);
        assert_eq!(header.header_data.chunk_size(), 2);
        assert_eq!(header.chunk_size_bytes(), 2048);

        // Test with values that should round up
        let header = NativeFileHeader::init(513, 8192);
        assert_eq!(header.header_data.chunk_size(), 1);
        assert_eq!(header.chunk_size_bytes(), 1024);

        let header = NativeFileHeader::init(1025, 8192);
        assert_eq!(header.header_data.chunk_size(), 2);
        assert_eq!(header.chunk_size_bytes(), 2048);

        // Test with a large value
        let header = NativeFileHeader::init(1_048_576, 8192); // 1 MB
        assert_eq!(header.header_data.chunk_size(), 11);
        assert_eq!(header.chunk_size_bytes(), 1_048_576);
    }
}
