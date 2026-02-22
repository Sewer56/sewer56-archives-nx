use crate::utilities::compression::{
    zstd::GetDecompressedSizeError, R3ACompressionError, R3ADecompressionError,
};
use core::{iter::once, str::from_utf8_unchecked};
use thiserror_no_std::Error;

/// Checks if a given path is present in the raw string pool data.
///
/// This function performs a linear search through the data.
/// It is case-sensitive and exact, meaning it will only return `true` if the
/// path is present in the pool exactly as specified.
///
/// # Arguments
/// * `raw_data` - The raw byte data of the string pool.
/// * `path` - The path to search for in the string pool.
///
/// # Returns
/// `true` if the path is present in the string pool, `false` otherwise.
pub fn contains(raw_data: &[u8], path: &str) -> bool {
    let path_bytes = path.as_bytes();
    raw_data
        .windows(path_bytes.len())
        .any(|window| window == path_bytes)
}

/// Returns the number of strings in the pool.
///
/// # Arguments
/// * `offsets` - The slice of offsets into the raw data.
///
/// # Returns
/// The number of strings in the pool.
pub fn len(offsets: &[u32]) -> usize {
    offsets.len()
}

/// Returns an iterator over the strings in the string pool.
///
/// # Arguments
/// * `raw_data` - The raw byte data of the string pool.
/// * `offsets` - The slice of offsets into the raw data.
///
/// # Returns
/// An iterator yielding string slices for each entry in the pool.
pub fn iter<'a>(raw_data: &'a [u8], offsets: &'a [u32]) -> impl Iterator<Item = &'a str> {
    offsets
        .windows(2)
        .map(move |window| {
            let start = window[0] as usize;
            let end = window[1] as usize;

            // SAFETY: The string pool is guaranteed to be valid UTF-8
            unsafe { from_utf8_unchecked(&raw_data[start..end]) }
        })
        .chain(once({
            let start = *offsets.last().unwrap() as usize;
            let end = raw_data.len();

            // SAFETY: The string pool is guaranteed to be valid UTF-8
            unsafe { from_utf8_unchecked(&raw_data[start..end]) }
        }))
}

/// Returns a string slice by index from the string pool.
///
/// # Arguments
/// * `raw_data` - The raw byte data of the string pool.
/// * `offsets` - The slice of offsets into the raw data.
/// * `index` - The index of the string to retrieve.
///
/// # Returns
/// A `Some(&str)` if the index is valid, or `None` if the index is out of bounds.
pub fn get<'a>(raw_data: &'a [u8], offsets: &'a [u32], index: usize) -> Option<&'a str> {
    if index >= offsets.len() {
        return None;
    }

    Some(unsafe { get_unchecked(raw_data, offsets, index) })
}

/// Returns a string slice by index from the string pool without bounds checking.
///
/// # Arguments
/// * `raw_data` - The raw byte data of the string pool.
/// * `offsets` - The slice of offsets into the raw data.
/// * `index` - The index of the string to retrieve.
///
/// # Returns
/// A `&str` slice for the given index.
///
/// # Safety
/// This function is unsafe because it does not perform bounds checking.
/// The caller must ensure that the index is within bounds.
/// It also assumes that the raw_data contains valid UTF-8.
pub unsafe fn get_unchecked<'a>(raw_data: &'a [u8], offsets: &'a [u32], index: usize) -> &'a str {
    let start = *offsets.get_unchecked(index) as usize;
    let end = if index + 1 < offsets.len() {
        *offsets.get_unchecked(index + 1) as usize
    } else {
        raw_data.len()
    };

    from_utf8_unchecked(&raw_data[start..end])
}

/// Represents an error obtained when trying to pack the string pool.
/// To see the format details, see the [`StringPoolFormat::V0`]
/// documentation.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StringPoolFormat {
    /// # String Pool (V0) Format
    ///
    /// The string pool is a flat buffer deduplicated strings UTF-8 of file paths.
    ///
    /// Each string is:
    /// - Null terminated
    /// - Uses '/' as separator on all platforms
    ///
    /// ***This is also the in-memory representation of this structure***
    ///
    /// # An Example
    ///
    ///  A valid (decompressed) pool might look like this:  
    /// `data/textures/cat.png\0data/textures/dog.png`
    ///
    /// String length is determined by searching null terminators. We will determine lengths of all strings
    /// ahead of time by scanning for (`0x00`) using SIMD. No edge cases; `0x00` is guaranteed null
    /// terminator due to nature of UTF-8 encoding.
    ///
    /// See UTF-8 encoding table:
    ///
    /// |  Code point range  |  Byte 1  |  Byte 2  |  Byte 3  |  Byte 4  | Code points |
    /// |:------------------:|:--------:|:--------:|:--------:|:--------:|:-----------:|
    /// |  U+0000 - U+007F   | 0xxxxxxx |          |          |          |     128     |
    /// |  U+0080 - U+07FF   | 110xxxxx | 10xxxxxx |          |          |    1920     |
    /// |  U+0800 - U+FFFF   | 1110xxxx | 10xxxxxx | 10xxxxxx |          |    61440    |
    /// | U+10000 - U+10FFFF | 11110xxx | 10xxxxxx | 10xxxxxx | 10xxxxxx |   1048576   |
    ///
    /// # Optimization
    ///
    /// The strings in this pool are first lexicographically sorted (to group similar paths together);
    /// and then compressed using ZStd. This improves compression ratio.
    ///
    /// The data is then compressed using non-streaming API, such that the
    /// ZStd frames contain the length info and the length can be determined with
    /// `ZSTD_findDecompressedSize`.
    ///
    /// The null terminators in this structure also help aid compression ratio; as they follow
    /// file extensions and therefore very often match multiple bytes back.
    V0,
}

/// Represents an error obtained when trying to pack the string pool.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StringPoolPackError {
    /// Compress pool exceeds maximum size limit.
    /// This means packing the R3A archive will most likely fail, so we bail out early.
    PoolTooLarge,

    /// Failed to compress pool.
    FailedToCompress(R3ACompressionError),

    /// The file path for the selected section format was too long.
    FilePathTooLong,
}

/// Represents an error obtained when trying to unpack the string pool.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum StringPoolUnpackError {
    /// Failed to decompress the pool contents.
    FailedToDecompress(#[from] R3ADecompressionError),

    /// Failed to determine decompressed size.
    FailedToGetDecompressedSize(#[from] GetDecompressedSizeError),

    /// Size of decompressed pool exceeds max limit;
    /// this is a measure to prevent servers from running out of memory
    /// due to malformed string pool size. Since you can't always trust user input.
    ExceededMaxSize(u32),

    /// Attempted buffer overflow detected.
    /// This usually happens if a decompressed string pool has less elements than it should.
    BufferOverflow,

    /// The StringPool should end on a null terminator, but it does not.
    ShouldEndOnNullTerminator,

    /// There is insufficient data to deserialize the string pool.
    NotEnoughData,
}
