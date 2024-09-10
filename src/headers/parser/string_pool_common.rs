use crate::utilities::compression::{
    zstd::GetDecompressedSizeError, NxCompressionError, NxDecompressionError,
};
use core::str::from_utf8_unchecked;
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
            let end = window[1] as usize - 1; // -1 to exclude null terminator

            // SAFETY: The string pool is guaranteed to be valid UTF-8
            unsafe { from_utf8_unchecked(&raw_data[start..end]) }
        })
        .chain(std::iter::once({
            let start = *offsets.last().unwrap() as usize;
            let end = raw_data.len() - 1; // -1 to exclude final null terminator

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
        *offsets.get_unchecked(index + 1) as usize - 1 // -1 to exclude null terminator
    } else {
        raw_data.len() - 1 // -1 to exclude final null terminator
    };

    from_utf8_unchecked(&raw_data[start..end])
}

/// Represents an error obtained when trying to pack the string pool.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StringPoolPackError {
    /// Compress pool exceeds maximum size limit.
    /// This means packing the Nx archive will most likely fail, so we bail out early.
    PoolTooLarge,

    /// Failed to compress pool.
    FailedToCompress(NxCompressionError),
}

/// Represents an error obtained when trying to unpack the string pool.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum StringPoolUnpackError {
    /// Failed to decompress the pool contents.
    FailedToDecompress(#[from] NxDecompressionError),

    /// Failed to determine decompressed size.
    FailedToGetDecompressedSize(#[from] GetDecompressedSizeError),
}
