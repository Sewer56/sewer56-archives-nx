use crate::utilities::compression::zstd::{
    self, compress_no_copy_fallback, GetDecompressedSizeError,
};
use crate::utilities::compression::{NxCompressionError, NxDecompressionError};
use crate::{
    api::traits::has_relative_path::HasRelativePath,
    headers::raw::native_toc_header::NativeTocHeader,
};
use core::marker::PhantomData;
use core::{mem::MaybeUninit, ptr::copy_nonoverlapping};
use std::alloc::{Allocator, Global, System};
use thiserror_no_std::Error;

/// The compression level used for the zstd stringpool.
/// This defaults to 16. Normally I would set this to 22,
/// however I found higher levels to not bring any space
/// savings in practice due to the nature of the data.
///
/// Very very rarely a higher level would save a byte or two.
const DEFAULT_COMPRESSION_LEVEL: i32 = 16;

/// Structure for serializing and deserializing the string pool within the Nx Archive format.
///
/// # Type Parameters
///
/// * `ShortAlloc` - Allocator for short lived memory. Think pooled memory and rentals.
/// * `LongAlloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive.
///
/// # String Pool Format
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
struct StringPool<ShortAlloc: Allocator + Clone = Global, LongAlloc: Allocator + Clone = Global> {
    /// The raw data of the string pool.
    /// This contains the null terminated strings.
    _raw_data: Box<[u8], LongAlloc>,

    /// The offsets into the raw data where the string data is located.
    /// These are raw byte offsets.
    _offsets: Box<[u32], LongAlloc>,

    _temp_allocator: PhantomData<ShortAlloc>,
    _comp_allocator: PhantomData<LongAlloc>,
}

impl StringPool {
    /// Packs a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `items` - The list of items to pack
    pub fn pack<T: HasRelativePath>(items: &mut [T]) -> Result<Vec<u8>, StringPoolPackError> {
        Self::pack_with_allocators(items, Global, Global)
    }
}

impl<ShortAlloc: Allocator + Clone, LongAlloc: Allocator + Clone>
    StringPool<ShortAlloc, LongAlloc>
{
    /// Packs a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `items` - The list of items to pack
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    pub fn pack_with_allocators<T: HasRelativePath>(
        items: &mut [T],
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
    ) -> Result<Vec<u8, LongAlloc>, StringPoolPackError> {
        items.sort_by(|a, b| a.relative_path().cmp(b.relative_path()));

        // Sum up all string lengths (incl. null terminators)
        let total_path_size: usize = items
            .iter()
            .map(|item| item.relative_path().len() + 1)
            .sum();

        // Allocate uninitialized memory
        let mut decompressed_pool: Box<[MaybeUninit<u8>], ShortAlloc> =
            Box::new_uninit_slice_in(total_path_size, short_alloc);

        let mut wat: Box<u8, &System> = Box::new_in(42, &System);

        let mut offset = 0;
        for item in items.iter() {
            let path = item.relative_path().as_bytes();
            let path_len = path.len();

            // Safety: We know exact length of pool before compression.
            // Manually copy the path bytes
            unsafe {
                copy_nonoverlapping(
                    path.as_ptr(),
                    decompressed_pool.as_mut_ptr().add(offset) as *mut u8,
                    path_len,
                );
            }

            // Add null terminator
            decompressed_pool[offset + path_len].write(0);
            offset += path_len + 1;
        }

        let decompressed_pool = unsafe { decompressed_pool.assume_init() };

        // Compress into destination
        let destination: Box<[MaybeUninit<u8>], LongAlloc> =
            Box::new_uninit_slice_in(total_path_size, long_alloc);
        let mut destination = unsafe { destination.assume_init() };
        let comp_result = compress_no_copy_fallback(
            DEFAULT_COMPRESSION_LEVEL,
            &decompressed_pool[..],
            &mut destination[..],
        );

        if let Err(x) = comp_result {
            return Err(StringPoolPackError::FailedToCompress(x));
        }

        if destination.len() <= NativeTocHeader::MAX_STRING_POOL_SIZE {
            Ok(destination.into_vec())
        } else {
            Err(StringPoolPackError::PoolTooLarge)
        }
    }

    /// Unpacks a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `source` - The compressed data to unpack.
    /// * `file_count` - Number of files in the archive. This is equal to number of entries.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    pub fn unpack_with_allocators(
        source: &[u8],
        file_count: usize,
        long_alloc: LongAlloc,
    ) -> Result<StringPool<ShortAlloc, LongAlloc>, StringPoolUnpackError> {
        // Determine size of decompressed data
        let decompressed_size = zstd::get_decompressed_size(source)?;

        // Decompress the data
        let decompressed = Box::new_uninit_slice_in(decompressed_size, long_alloc.clone());
        let mut decompressed = unsafe { decompressed.assume_init() };
        zstd::decompress(source, &mut decompressed[..])?;
        // Populate all offsets
        let str_offsets: Box<[MaybeUninit<u32>], LongAlloc> =
            Box::new_uninit_slice_in(file_count, long_alloc.clone());
        let mut str_offsets = unsafe { str_offsets.assume_init() };

        let mut current_offset: u32 = 0;
        let mut current_str_offset = str_offsets.as_mut_ptr(); // SAFETY: str_offsets is file_count in length
        for _ in 0..file_count - 1 {
            unsafe { *current_str_offset = current_offset };
            let file_length = 0;
            current_offset += file_length;
            current_str_offset = unsafe { current_str_offset.add(1) };
        }

        Ok(StringPool {
            _offsets: str_offsets,
            _raw_data: decompressed,
            _temp_allocator: PhantomData,
            _comp_allocator: PhantomData,
        })
    }
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
