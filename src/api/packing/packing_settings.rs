// STD ALERT!! However it's portable traits only.
use crate::{api::enums::*, utilities::system_info::get_num_cores};
use core::num::NonZeroU32;
use std::io::{Seek, Write};

/// The minimum block size that the user is allowed to specify
pub const MIN_BLOCK_SIZE: u32 = 4095;

/// The maximum block size that the user is allowed to specify
pub const MAX_BLOCK_SIZE: u32 = 67_108_863;

/// The minimum chunk size that the user is allowed to specify
pub const MIN_CHUNK_SIZE: u32 = 32_768;

/// The maximum chunk size that the user is allowed to specify
pub const MAX_CHUNK_SIZE: u32 = 1_073_741_824;

/// Controls the configuration settings of the packer.
///
/// # Remarks
///
/// This struct contains settings that determine how the packing process
/// will be performed, including block and chunk sizes, compression levels,
/// and compression algorithms.
pub struct PackingSettings<W: Write + Seek> {
    /// The stream to which data is output to.
    /// This stream must support seeking.
    ///
    /// # Remarks
    /// This assumes the stream starts at offset 0.
    /// If you need the ability to write to a middle of an existing stream, raise a PR.
    pub output: W,

    /// Maximum number of threads allowed.
    pub max_num_threads: NonZeroU32,

    /// Size of SOLID blocks.\
    /// Range is MIN_BLOCK_SIZE to 67108863 (64 MiB).\
    /// Values are powers of 2, minus 1.\
    ///
    /// Must be smaller than [`Self::chunk_size`].
    pub block_size: u32,

    /// Size of large file chunks.
    ///
    /// Range is 32768 (32K) to 1073741824 (1 GiB).\
    /// Values are powers of 2.
    ///
    /// Must be greater than [`Self::block_size`].
    pub chunk_size: u32,

    /// Set this to 'true' to store hashes in the ToC.
    /// Without this, hashes will not be stored in the ToC.
    pub store_hashes: bool,

    /// Compression level to use for SOLID data.
    ///
    /// # Range
    ///
    /// ZStandard has Range -5 - 22.\
    /// LZ4 has Range: 1 - 12.
    pub solid_compression_level: i32,

    /// Compression level to use for chunked data.
    ///
    /// # Range
    ///
    /// ZStandard has Range -5 - 22.\
    /// LZ4 has Range: 1 - 12.
    pub chunked_compression_level: i32,

    /// Compression algorithm used for compressing SOLID blocks.
    pub solid_block_algorithm: CompressionPreference,

    /// Compression algorithm used for compressing chunked files.
    pub chunked_file_algorithm: CompressionPreference,

    /// Enables deduplication of chunks. If true, chunks are deduplicated.
    /// Chunk deduplication encurs a small amount of overhead for each file.
    pub enable_chunked_deduplication: bool,

    /// Enables deduplication of chunks. If true, chunks are deduplicated.
    /// Solid deduplication is virtually free for each file
    pub enable_solid_deduplication: bool,
}

impl<W: Write + Seek> PackingSettings<W> {
    /// Creates a new `PackingSettings` with default values.
    pub fn new(output: W) -> Self {
        PackingSettings {
            output,
            max_num_threads: get_num_cores(),
            block_size: 1_048_575,
            chunk_size: 1_048_576,
            solid_compression_level: 16,
            chunked_compression_level: 9,
            solid_block_algorithm: CompressionPreference::ZStandard,
            chunked_file_algorithm: CompressionPreference::ZStandard,
            enable_chunked_deduplication: false,
            enable_solid_deduplication: true,
            store_hashes: true,
        }
    }

    /// Sanitizes settings to acceptable values if they are out of range or undefined.
    pub fn sanitize(&mut self) {
        // Note: BlockSize is minus one, see spec.
        self.block_size = self.block_size.clamp(MIN_BLOCK_SIZE, MAX_BLOCK_SIZE);
        // 1GiB because larger chunks cause problems with LZ4 and the likes
        self.chunk_size = self.chunk_size.clamp(MIN_CHUNK_SIZE, MAX_CHUNK_SIZE);

        self.block_size = self.block_size.next_power_of_two() - 1;
        self.chunk_size = self.chunk_size.next_power_of_two();

        if self.chunk_size <= self.block_size {
            self.chunk_size = self.block_size + 1;
        }

        self.solid_compression_level =
            self.clamp_compression(self.solid_compression_level, &self.solid_block_algorithm);
        self.chunked_compression_level =
            self.clamp_compression(self.chunked_compression_level, &self.chunked_file_algorithm);
        self.max_num_threads = self
            .max_num_threads
            .clamp(unsafe { NonZeroU32::new_unchecked(1) }, NonZeroU32::MAX);
    }

    /// Retrieves the compression level for the specified algorithm.
    fn clamp_compression(&self, level: i32, preference: &CompressionPreference) -> i32 {
        match preference {
            CompressionPreference::Copy => 1,
            CompressionPreference::ZStandard => level.clamp(-5, 22),
            CompressionPreference::Lz4 => level.clamp(1, 12),
            CompressionPreference::NoPreference => 1,
        }
    }
}

// Unit tests using rstest
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::io::Cursor;

    #[rstest(chunk_size, expected,
        case(1_073_741_825u32, MAX_CHUNK_SIZE), // Exceeds max chunk size, clamped down
        case(u32::MAX, MAX_CHUNK_SIZE),         // Max u32 value, clamped to max chunk size
        case(0u32, MIN_CHUNK_SIZE)              // Zero value, adjusted to min chunk size
    )]
    fn chunk_size_is_clamped(chunk_size: u32, expected: u32) {
        let output = Cursor::new(Vec::new());
        let mut settings = PackingSettings::new(output);
        settings.chunk_size = chunk_size;
        settings.block_size = MIN_BLOCK_SIZE; // Set block_size to minimum to avoid influencing chunk_size
        settings.sanitize();
        assert_eq!(settings.chunk_size, expected);
    }

    #[rstest(value, expected,
        case(MAX_BLOCK_SIZE + 1, MAX_BLOCK_SIZE), // Exceeds max block size, clamped down
        case(u32::MAX, MAX_BLOCK_SIZE),           // Max u32 value, clamped to max block size
        case(MIN_BLOCK_SIZE - 1, MIN_BLOCK_SIZE), // Below minimum, adjusted to min block size
        case(0u32, MIN_BLOCK_SIZE)                // Zero value, adjusted to min block size
    )]
    fn block_size_is_clamped(value: u32, expected: u32) {
        let output = Cursor::new(Vec::new());
        let mut settings = PackingSettings::new(output);
        settings.block_size = value;
        settings.sanitize();
        assert_eq!(settings.block_size, expected);
    }

    #[rstest(block_size, chunk_size,
        // Regular Values
        case(32_767u32, 4_194_304u32),            // Valid block and chunk sizes
        case(MAX_BLOCK_SIZE, MAX_BLOCK_SIZE + 1), // Max block size and valid chunk size
        case(MIN_BLOCK_SIZE - 1, MIN_CHUNK_SIZE), // Minimum block and chunk sizes
        case(67_108_862u32, MAX_BLOCK_SIZE),      // Just below max sizes

        // BlockSize > ChunkSize (should adjust chunk_size)
        case(MAX_BLOCK_SIZE, 4_194_304u32), // Block size exceeds chunk size
        case(4_194_305u32, 4_194_304u32),   // Block size slightly larger than chunk size
        case(MAX_BLOCK_SIZE, 67_108_862u32) // Block size one more than chunk size
    )]
    fn chunk_size_must_be_greater_than_block_size(block_size: u32, chunk_size: u32) {
        let output = Cursor::new(Vec::new());
        let mut settings = PackingSettings::new(output);
        settings.block_size = block_size;
        settings.chunk_size = chunk_size;
        settings.sanitize();
        assert!(settings.chunk_size > settings.block_size);
    }

    #[rstest(value, expected,
        case(23, 22),        // Above max ZStandard level, clamped to 22
        case(i32::MAX, 22),  // Max i32 value, clamped to 22
        case(0, 0),          // Valid ZStandard level, remains unchanged
        case(i32::MIN, -5)   // Below min ZStandard level, clamped to -5
    )]
    fn zstandard_level_is_clamped(value: i32, expected: i32) {
        let output = Cursor::new(Vec::new());
        let mut settings = PackingSettings::new(output);
        settings.solid_compression_level = value;
        settings.solid_block_algorithm = CompressionPreference::ZStandard;
        settings.sanitize();
        assert_eq!(settings.solid_compression_level, expected);
    }

    #[rstest(value, expected,
        case(13, 12),        // Above max LZ4 level, clamped to 12
        case(i32::MAX, 12),  // Max i32 value, clamped to 12
        case(0, 1),          // Below min LZ4 level, clamped to 1
        case(i32::MIN, 1)    // Below min LZ4 level, clamped to 1
    )]
    fn lz4_level_is_clamped(value: i32, expected: i32) {
        let output = Cursor::new(Vec::new());
        let mut settings = PackingSettings::new(output);
        settings.chunked_file_algorithm = CompressionPreference::Lz4;
        settings.chunked_compression_level = value;
        settings.sanitize();
        assert_eq!(settings.chunked_compression_level, expected);
    }

    #[rstest(value, expected,
        case(NonZeroU32::MAX, NonZeroU32::MAX), // Max number of threads, remains unchanged
        // Stays at min value
        case(unsafe { NonZeroU32::new_unchecked(1) } , unsafe { NonZeroU32::new_unchecked(1) })
        // Negative values are not possible for usize
    )]
    fn max_num_threads_is_clamped(value: NonZeroU32, expected: NonZeroU32) {
        let output = Cursor::new(Vec::new());
        let mut settings = PackingSettings::new(output);
        settings.max_num_threads = value;
        settings.sanitize();
        assert_eq!(settings.max_num_threads, expected);
    }

    #[test]
    fn deduplication_flags_default_values() {
        let output = Cursor::new(Vec::new());
        let settings = PackingSettings::new(output);
        assert!(!settings.enable_chunked_deduplication);
        assert!(settings.enable_solid_deduplication);
    }
}
