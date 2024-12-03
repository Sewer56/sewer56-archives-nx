use super::string_pool_common::{
    self, StringPoolFormat, StringPoolPackError, StringPoolUnpackError,
};
use crate::api::traits::*;
use crate::headers::raw::toc::*;
use crate::utilities::compression::zstd::{
    self, compress_no_copy_fallback, max_alloc_for_compress_size,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::write_bytes;
use core::{mem::MaybeUninit, ptr::copy_nonoverlapping};
use memchr::Memchr;
use std::alloc::{Allocator, Global};

/// The compression level used for the zstd stringpool.
/// This defaults to 16. Normally I would set this to 22,
/// however I found higher levels to not bring any space
/// savings in practice due to the nature of the data.
///
/// Levels beyond this point don't save much space.
const DEFAULT_COMPRESSION_LEVEL: i32 = 16;

/// Structure for serializing and deserializing the string pool within the Nx Archive format.
///
/// # Type Parameters
///
/// * `ShortAlloc` - Allocator for short lived memory. Think pooled memory and rentals.
/// * `LongAlloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive.
///
pub struct StringPool<ShortAlloc: Allocator + Clone = Global, LongAlloc: Allocator + Clone = Global>
{
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
    /// * `format` - The format of the string pool
    /// * `use_compression` - Whether to compress the string pool.
    ///                       This is only set to 'false' in tests to skip non-rust code under 'miri', and benchmarks.
    ///                       In actual archive use this is always 'true'.
    pub fn pack<T: HasRelativePath>(
        items: &mut [T],
        format: StringPoolFormat,
        use_compression: bool,
    ) -> Result<Vec<u8>, StringPoolPackError> {
        match format {
            StringPoolFormat::V0 => Self::pack_v0(items, use_compression),
        }
    }

    /// Packs a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `items` - The list of items to pack
    /// * `use_compression` - Whether to compress the string pool.
    ///                       This is only set to 'false' in tests to skip non-rust code under 'miri', and benchmarks.
    ///                       In actual archive use this is always 'true'.
    pub fn pack_v0<T: HasRelativePath>(
        items: &mut [T],
        use_compression: bool,
    ) -> Result<Vec<u8>, StringPoolPackError> {
        Self::pack_v0_with_allocators(items, Global, Global, use_compression)
    }

    /// Unpacks a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `source` - The compressed data to unpack.
    /// * `file_count` - Number of files in the archive. This is equal to number of entries.
    /// * `format` - The (file) format of the string pool
    /// * `use_compression` - Whether to compress the string pool.
    ///                       This is only set to 'false' in tests to skip non-rust code under 'miri', and benchmarks.
    ///                       In actual archive use this is always 'true'.
    pub fn unpack(
        source: &[u8],
        file_count: usize,
        format: StringPoolFormat,
        use_compression: bool,
    ) -> Result<Self, StringPoolUnpackError> {
        match format {
            StringPoolFormat::V0 => Self::unpack_v0(source, file_count, use_compression),
        }
    }

    /// Unpacks a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `source` - The compressed data to unpack.
    /// * `file_count` - Number of files in the archive. This is equal to number of entries.
    /// * `use_compression` - Whether to compress the string pool.
    ///                       This is only set to 'false' in tests to skip non-rust code under 'miri', and benchmarks.
    ///                       In actual archive use this is always 'true'.
    pub fn unpack_v0(
        source: &[u8],
        file_count: usize,
        use_compression: bool,
    ) -> Result<Self, StringPoolUnpackError> {
        Self::unpack_v0_with_allocators(source, file_count, Global, Global, use_compression)
    }
}

impl<ShortAlloc: Allocator + Clone, LongAlloc: Allocator + Clone>
    StringPool<ShortAlloc, LongAlloc>
{
    /// Checks if a given path is present in the string pool.
    ///
    /// This function performs a linear search through the string pool's data.
    /// It is case-sensitive and exact, meaning it will only return `true` if the
    /// path is present in the pool exactly as specified.
    ///
    /// # Arguments
    /// * `path` - The path to search for in the string pool.
    ///
    /// # Returns
    /// `true` if the path is present in the string pool, `false` otherwise.
    pub fn contains(&self, path: &str) -> bool {
        string_pool_common::contains(&self._raw_data, path)
    }

    /// Returns the number of items in the string pool.
    ///
    /// # Returns
    /// The number of items in the string pool.
    ///
    /// # Remarks
    ///
    /// This function simply returns the length of the string pool's internal
    /// offset array, which corresponds to the number of items stored in the pool.
    pub fn len(&self) -> usize {
        string_pool_common::len(&self._offsets)
    }

    /// Returns whether the pool is empty.
    pub fn is_empty(&self) -> bool {
        string_pool_common::len(&self._offsets) == 0
    }

    /// Returns an iterator over the strings in the string pool.
    ///
    /// This iterator yields references to the strings in the pool, in the order
    /// they were inserted. The strings are not cloned or copied, so the iterator
    /// is efficient and does not allocate any additional memory.
    ///
    /// # Returns
    ///
    /// An iterator over the strings in the string pool.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        string_pool_common::iter(&self._raw_data, &self._offsets)
    }

    /// Returns a string slice by index from the string pool.
    ///
    /// # Arguments
    /// * `index` - The index of the string to retrieve.
    ///
    /// # Returns
    /// A `&str` slice if the index is valid, or `None` if the index is out of bounds.
    ///
    /// # Safety
    /// This function uses `from_utf8_unchecked` because the string pool is guaranteed
    /// to contain valid UTF-8.
    pub fn get(&self, index: usize) -> Option<&str> {
        string_pool_common::get(&self._raw_data, &self._offsets, index)
    }

    /// Returns a string slice by index from the string pool without bounds checking.
    ///
    /// # Arguments
    /// * `index` - The index of the string to retrieve.
    ///
    /// # Returns
    /// A `&str` slice for the given index.
    ///
    /// # Safety
    /// This function is unsafe because it does not perform bounds checking.
    /// The caller must ensure that the index is within bounds.
    /// It also uses `from_utf8_unchecked` because the string pool is guaranteed
    /// to contain valid UTF-8.
    pub unsafe fn get_unchecked(&self, index: usize) -> &str {
        string_pool_common::get_unchecked(&self._raw_data, &self._offsets, index)
    }

    /// Packs a list of items into a string pool in its native binary format, using custom allocators.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `items` - The list of items to pack
    /// * `format` - The format of the string pool
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    /// * `use_compression` - Whether to compress the string pool.
    ///                       This is only set to 'false' in tests to skip non-rust code under 'miri', and benchmarks.
    ///                       In actual archive use this is always 'true'.
    pub fn pack_with_allocators<T: HasRelativePath>(
        items: &mut [T],
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
        format: StringPoolFormat,
        use_compression: bool,
    ) -> Result<Vec<u8, LongAlloc>, StringPoolPackError> {
        match format {
            StringPoolFormat::V0 => {
                Self::pack_v0_with_allocators(items, short_alloc, long_alloc, use_compression)
            }
        }
    }

    /// Unpacks a list of items into a string pool in its native binary format, using custom allocators.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `source` - The compressed data to unpack.
    /// * `file_count` - Number of files in the archive. This is equal to number of entries.
    /// * `format` - The (file) format of the string pool
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    /// * `use_compression` - Whether to compress the string pool.
    ///                       This is only set to 'false' in tests to skip non-rust code under 'miri', and benchmarks.
    ///                       In actual archive use this is always 'true'.
    pub fn unpack_with_allocators(
        source: &[u8],
        file_count: usize,
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
        format: StringPoolFormat,
        use_compression: bool,
    ) -> Result<StringPool<ShortAlloc, LongAlloc>, StringPoolUnpackError> {
        match format {
            StringPoolFormat::V0 => Self::unpack_v0_with_allocators(
                source,
                file_count,
                short_alloc,
                long_alloc,
                use_compression,
            ),
        }
    }

    /// Packs a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `items` - The list of items to pack
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    /// * `use_compression` - Whether to compress the string pool.
    ///                       This is only set to 'false' in tests to skip non-rust code under 'miri', and benchmarks.
    ///                       In actual archive use this is always 'true'.
    ///
    /// # Remarks
    ///
    /// For the file format details, see the [StringPoolFormat::V0] documentation.
    pub fn pack_v0_with_allocators<T: HasRelativePath>(
        items: &mut [T],
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
        use_compression: bool,
    ) -> Result<Vec<u8, LongAlloc>, StringPoolPackError> {
        items.sort_by(|a, b| a.relative_path().cmp(b.relative_path()));

        // Sum up all string lengths (incl. null terminators)
        let total_size = calc_pool_size(items);

        // Allocate uninitialized memory
        let mut decompressed_pool: Box<[MaybeUninit<u8>], ShortAlloc> =
            Box::new_uninit_slice_in(total_size, short_alloc);

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
            // SAFETY: We know decompressed_pool is long enough based on the assumption calc_pool_size
            // is correct, which it is (passes miri).
            unsafe {
                *decompressed_pool.as_mut_ptr().add(offset + path_len) = MaybeUninit::new(0);
            }
            offset += path_len + 1;
        }

        let decompressed_pool = unsafe { decompressed_pool.assume_init() };
        if use_compression {
            Self::compress_pool(&decompressed_pool, long_alloc)
        } else {
            // This path is unoptimized in grand scheme of things, because it's only used for testing.
            let mut result = Vec::with_capacity_in(total_size, long_alloc);
            unsafe {
                copy_nonoverlapping(decompressed_pool.as_ptr(), result.as_mut_ptr(), total_size);
                result.set_len(total_size);
            }
            Ok(result)
        }
    }

    /// Unpacks a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `source` - The compressed data to unpack.
    /// * `file_count` - Number of files in the archive. This is equal to number of entries.
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    /// * `use_compression` - Whether to compress the string pool.
    ///                       This is only set to 'false' in tests to skip non-rust code under 'miri', and benchmarks.
    ///                       In actual archive use this is always 'true'.
    ///
    /// # Remarks
    ///
    /// For the file format details, see the [StringPoolFormat::V0] documentation.
    pub fn unpack_v0_with_allocators(
        source: &[u8],
        file_count: usize,
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
        use_compression: bool,
    ) -> Result<StringPool<ShortAlloc, LongAlloc>, StringPoolUnpackError> {
        // If there is no data, return an empty pool.
        // This is a fast return, in practice the library should never generate this case,
        // but it's technically valid per spec, since spec has length of compressed string pool
        // as a field.
        if source.is_empty() {
            return return_empty_pool(&long_alloc);
        }

        let decompressed_size;
        let decompressed: Box<[u8], ShortAlloc> = if use_compression {
            // Determine size of decompressed data
            // Note: This is very fast `O(1)` because the zstd frame header will contain the necessary info.
            decompressed_size = zstd::get_decompressed_size(source)?;

            // SAFETY: Compressed data is empty or zstd frame is missing size, return an empty pool.
            if decompressed_size == 0 {
                return return_empty_pool(&long_alloc);
            }

            // SAFETY: Don't trust user input; in case Nx is being ran on a server.
            //         If the frame size exceeds our allowed limit, return an error.
            if decompressed_size > MAX_STRING_POOL_SIZE {
                return Err(StringPoolUnpackError::ExceededMaxSize(
                    MAX_STRING_POOL_SIZE as u32,
                ));
            }

            // Decompress the data
            let mut decompressed =
                unsafe { Box::new_uninit_slice_in(decompressed_size, short_alloc).assume_init() };
            zstd::decompress(source, &mut decompressed[..])?;
            decompressed
        } else {
            // For uncompressed data, we can directly use the source
            decompressed_size = source.len();
            let mut decompressed = Box::new_uninit_slice_in(source.len(), short_alloc);
            unsafe {
                copy_nonoverlapping(
                    source.as_ptr(),
                    decompressed.as_mut_ptr() as *mut u8,
                    source.len(),
                );
                decompressed.assume_init()
            }
        };

        // Validate the decompressed segment ends with a null terminator
        #[cfg(feature = "hardened")]
        if decompressed.len() > 0 && decompressed[decompressed.len() - 1] != 0 {
            return Err(StringPoolUnpackError::ShouldEndOnNullTerminator);
        }

        // Populate all offsets
        let mut str_offsets: Box<[u32], LongAlloc> =
            unsafe { Box::new_uninit_slice_in(file_count, long_alloc.clone()).assume_init() };

        // Allocate space for paths without null terminators
        let mut raw_data: Box<[u8], LongAlloc> = unsafe {
            Box::new_uninit_slice_in(decompressed_size - file_count, long_alloc.clone())
                .assume_init()
        };

        // TODO: https://github.com/BurntSushi/memchr/issues/160
        // Add compile-time substitution.
        let mut memchr_iter = Memchr::new(0, &decompressed);
        let mut last_start_offset = 0; // Offset into the decompressed data
        let mut dest_copy_offset = 0; // Offset where we copy into the raw data
        let mut file_idx = 0;

        while file_idx < file_count {
            // SAFETY: It's not possible to overflow str_offsets here, because len of str_offsets'
            // array length equals `file_count`.
            unsafe {
                *str_offsets.get_mut(file_idx).unwrap_unchecked() = dest_copy_offset;
            }
            file_idx += 1;

            if let Some(null_pos) = memchr_iter.next() {
                let len = null_pos - last_start_offset;

                #[cfg(feature = "hardened")]
                {
                    // Ensure we don't exceed the allocated space
                    if dest_copy_offset as usize + len > raw_data.len() {
                        return Err(StringPoolUnpackError::BufferOverflow);
                    }
                }

                unsafe {
                    // SAFETY: memchr_iter ensures we don't overrun here.
                    copy_nonoverlapping(
                        decompressed.as_ptr().add(last_start_offset),
                        raw_data.as_mut_ptr().add(dest_copy_offset as usize),
                        len,
                    );
                }

                dest_copy_offset += len as u32;
                last_start_offset = null_pos + 1; // +1 to skip the null terminator
            } else {
                // If we've reached the end of the data, break the loop

                // Check if there's remaining data to process
                let remaining_len = decompressed_size - last_start_offset - 1;
                #[cfg(feature = "hardened")]
                {
                    if dest_copy_offset as usize + remaining_len > raw_data.len() {
                        return Err(StringPoolUnpackError::BufferOverflow);
                    }
                }

                unsafe {
                    // SAFETY: count cannot exceed decompressed_size since source_offset is positive
                    copy_nonoverlapping(
                        decompressed.as_ptr().add(last_start_offset),
                        raw_data.as_mut_ptr().add(dest_copy_offset as usize),
                        remaining_len,
                    );
                }

                dest_copy_offset += remaining_len as u32;
                break;
            }
        }

        // SAFETY: If the input had strings beyond the end of the expected count, then raw_data
        // will have uninitialized memory. In this case, we must write into that memory, as there's
        // technically non-zero chance there will be data that may make calls like `contains` invalid.
        // Thanks miri <3
        let remaining_bytes = raw_data.len() - dest_copy_offset as usize;
        if remaining_bytes > 0 {
            unsafe {
                // SAFETY: dest_copy_offset is less than raw_data.len()
                write_bytes(
                    raw_data.as_mut_ptr().add(dest_copy_offset as usize),
                    0,
                    remaining_bytes,
                );
            }
        }

        Ok(StringPool {
            _offsets: str_offsets,
            _raw_data: raw_data,
            _temp_allocator: PhantomData,
            _comp_allocator: PhantomData,
        })
    }

    fn compress_pool(
        decompressed_pool: &[u8],
        long_alloc: LongAlloc,
    ) -> Result<Vec<u8, LongAlloc>, StringPoolPackError> {
        let destination: Box<[MaybeUninit<u8>], LongAlloc> = Box::new_uninit_slice_in(
            max_alloc_for_compress_size(decompressed_pool.len()),
            long_alloc,
        );
        let mut destination = unsafe { destination.assume_init() };
        let comp_result = compress_no_copy_fallback(
            DEFAULT_COMPRESSION_LEVEL,
            decompressed_pool,
            &mut destination[..],
        );

        match comp_result {
            Ok(num_bytes) => {
                if destination.len() <= MAX_STRING_POOL_SIZE {
                    let mut vec = destination.into_vec();
                    // SAFETY: We know exact length of pool after compression, if it did not fit, we would have matched the error branch.
                    unsafe { vec.set_len(num_bytes) };
                    Ok(vec)
                } else {
                    Err(StringPoolPackError::PoolTooLarge)
                }
            }
            Err(x) => Err(StringPoolPackError::FailedToCompress(x)),
        }
    }
}

fn return_empty_pool<ShortAlloc: Allocator + Clone, LongAlloc: Allocator + Clone>(
    long_alloc: &LongAlloc,
) -> Result<StringPool<ShortAlloc, LongAlloc>, StringPoolUnpackError> {
    let str_offsets: Box<[u32], LongAlloc> =
        unsafe { Box::new_uninit_slice_in(0, long_alloc.clone()).assume_init() };
    let raw_data: Box<[u8], LongAlloc> =
        unsafe { Box::new_uninit_slice_in(0, long_alloc.clone()).assume_init() };

    Ok(StringPool {
        _offsets: str_offsets,
        _raw_data: raw_data,
        _temp_allocator: PhantomData,
        _comp_allocator: PhantomData,
    })
}

/// Calculates the total size of the pool for both the
/// [`StringPoolFormat::V0`] and [`StringPoolFormat::V1`] formats.
fn calc_pool_size<T: HasRelativePath>(items: &mut [T]) -> usize {
    let total_path_size: usize = items
        .iter()
        .map(|item| item.relative_path().len())
        .sum::<usize>()
        + items.len();
    total_path_size
}

#[cfg(test)]
mod tests {
    use crate::headers::raw::toc::*;
    use crate::utilities::compression::zstd::compress_no_copy_fallback;
    use crate::utilities::compression::NxDecompressionError;
    use crate::{
        api::traits::*,
        headers::parser::{
            string_pool::{StringPool, StringPoolUnpackError},
            string_pool_common::StringPoolFormat::{self, *},
        },
    };
    use alloc::format;
    use alloc::vec;
    use alloc::{string::String, vec::Vec};
    use rstest::rstest;
    use std::alloc::System;

    #[derive(Debug, PartialEq, Eq)]
    struct TestItem {
        path: String,
    }

    impl HasRelativePath for TestItem {
        fn relative_path(&self) -> &str {
            &self.path
        }
    }

    #[rstest]
    #[cfg_attr(not(miri), case(V0, true))]
    #[case(V0, false)]
    fn can_pack_and_unpack(#[case] format: StringPoolFormat, #[case] use_compression: bool) {
        let mut items: Vec<TestItem> = vec![
            TestItem {
                path: "data/textures/cat.png".to_string(),
            },
            TestItem {
                path: "data/textures/dog.png".to_string(),
            },
            TestItem {
                path: "data/models/house.obj".to_string(),
            },
        ];

        let packed = StringPool::pack(&mut items, format, use_compression).unwrap();
        let unpacked = StringPool::unpack(&packed, items.len(), format, use_compression).unwrap();

        // Check if the unpacked string pool contains all original items
        for item in &items {
            assert!(unpacked.contains(&item.path));
        }

        // Verify the order of items (should be lexicographically sorted)
        let sorted_paths: Vec<&str> = unpacked.iter().collect();
        assert_eq!(
            sorted_paths,
            vec![
                "data/models/house.obj",
                "data/textures/cat.png",
                "data/textures/dog.png"
            ]
        );
    }

    #[rstest]
    #[case(V0)]
    #[cfg_attr(miri, ignore)]
    fn can_pack_empty_list(#[case] format: StringPoolFormat) {
        let mut items: Vec<TestItem> = Vec::new();
        let packed = StringPool::pack(&mut items, format, true).unwrap();
        assert!(!packed.is_empty()); // Even an empty pool should have some metadata

        let unpacked = StringPool::unpack(&packed, 0, format, true).unwrap();
        assert_eq!(unpacked.len(), 0);
    }

    #[rstest]
    #[case(V0, true)]
    #[cfg_attr(miri, ignore)]
    fn can_pack_large_list(#[case] format: StringPoolFormat, #[case] use_compression: bool) {
        let mut items: Vec<TestItem> = (0..10000)
            .map(|i| TestItem {
                path: format!("file_{:05}.txt", i),
            })
            .collect();

        let packed = StringPool::pack(&mut items, format, use_compression).unwrap();
        let unpacked = StringPool::unpack(&packed, items.len(), format, use_compression).unwrap();

        assert_eq!(unpacked.len(), items.len());
        (0..unpacked.len())
            .for_each(|x| unsafe { assert_eq!(items[x].path, unpacked.get_unchecked(x)) });
    }

    #[rstest]
    #[case(V0)]
    #[cfg_attr(miri, ignore)]
    fn unpack_invalid_data(#[case] format: StringPoolFormat) {
        let invalid_data = vec![0, 1, 2, 3, 4]; // Invalid compressed data
        let result = StringPool::unpack(&invalid_data, 1, format, true);
        assert!(matches!(
            result,
            Err(StringPoolUnpackError::FailedToGetDecompressedSize(_)
                | StringPoolUnpackError::FailedToDecompress(_))
        ));
    }

    #[rstest]
    #[case(V0)]
    #[cfg_attr(miri, ignore)]
    fn pack_with_custom_allocators(#[case] format: StringPoolFormat) {
        let mut items = vec![
            TestItem {
                path: "data/textures/cat.png".to_string(),
            },
            TestItem {
                path: "data/textures/dog.png".to_string(),
            },
        ];

        let packed =
            StringPool::pack_with_allocators(&mut items, System, System, format, true).unwrap();
        let unpacked =
            StringPool::unpack_with_allocators(&packed, items.len(), System, System, format, true)
                .unwrap();

        assert_eq!(unpacked.len(), items.len());
        for item in &items {
            assert!(unpacked.contains(&item.path));
        }
    }

    #[rstest]
    #[case(false)]
    #[cfg_attr(not(miri), case(true))]
    fn v0_can_use_paths_over_256chars(#[case] use_compression: bool) {
        let mut items = vec![
            TestItem {
                // Exceeds 256 chars
                path: "/".to_owned() + &"a".repeat(255) + "/file.txt",
            },
            TestItem {
                path: "data/textures/cat.png".to_string(),
            },
        ];

        let packed = StringPool::pack(&mut items, V0, use_compression).unwrap();
        let unpacked = StringPool::unpack(&packed, items.len(), V0, use_compression).unwrap();

        assert_eq!(unpacked.len(), items.len());
        for item in &items {
            assert!(unpacked.contains(&item.path));
        }
    }

    #[rstest]
    #[cfg_attr(not(miri), case(V0, true))]
    #[case(V0, false)]
    fn can_use_non_ascii_paths(#[case] format: StringPoolFormat, #[case] use_compression: bool) {
        let mut items = vec![
            TestItem {
                path: "data/textures/猫.png".to_string(),
            },
            TestItem {
                path: "data/models/家.obj".to_string(),
            },
            TestItem {
                path: "data/音楽/曲.mp3".to_string(),
            },
        ];

        let packed = StringPool::pack(&mut items, format, use_compression).unwrap();
        let unpacked = StringPool::unpack(&packed, items.len(), format, use_compression).unwrap();

        assert_eq!(unpacked.len(), items.len());
        for item in &items {
            assert!(unpacked.contains(&item.path));
        }

        // Check lexicographic ordering of non-ASCII paths
        let paths: Vec<&str> = unpacked.iter().collect();
        assert_eq!(
            paths,
            vec![
                "data/models/家.obj",
                "data/textures/猫.png",
                "data/音楽/曲.mp3"
            ]
        );
    }

    #[rstest]
    #[case(StringPoolFormat::V0)]
    #[cfg_attr(miri, ignore)]
    fn unpack_fails_when_zstd_frame_size_exceeds_max(#[case] format: StringPoolFormat) {
        // Create a large input that exceeds MAX_STRING_POOL_SIZE
        let large_input = vec![b'A'; MAX_STRING_POOL_SIZE + 1];

        // Compress the large input
        let mut compressed = vec![0u8; MAX_STRING_POOL_SIZE + 1];
        let comp_result = compress_no_copy_fallback(1, &large_input, &mut compressed).unwrap();
        compressed.truncate(comp_result);

        // Attempt to unpack the compressed data
        let result = StringPool::unpack(&compressed, 1, format, true);

        // Check that the result is an error and specifically an ExceededMaxSize error
        assert!(matches!(
            result,
            Err(StringPoolUnpackError::ExceededMaxSize(size)) if size == MAX_STRING_POOL_SIZE as u32
        ));
    }

    #[rstest]
    #[case(StringPoolFormat::V0)]
    #[cfg_attr(miri, ignore)]
    fn unpack_fails_when_frame_size_missing(#[case] format: StringPoolFormat) {
        // Pre-compressed "Hello, World!" without frame size
        let no_frame_size = vec![
            0x28, 0xB5, 0x2F, 0xFD, 0x04, 0x00, 0x41, 0x10, 0x00, 0x00, 0x48, 0x65, 0x6C, 0x6C,
            0x6F, 0x2C, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x03,
        ];

        let result = StringPool::unpack(&no_frame_size, 1, format, true);
        assert!(matches!(
            result,
            Err(StringPoolUnpackError::FailedToGetDecompressedSize(_))
        ));
    }

    #[rstest]
    #[case(StringPoolFormat::V0)]
    #[cfg_attr(miri, ignore)]
    fn unpack_fails_when_zstd_frame_size_made_too_small(#[case] format: StringPoolFormat) {
        // Compressed data with altered frame size 86 -> 40 bytes.
        // Data is following text:
        //
        //      Hello World!!
        //      I am an evil file, my frame size is too small compared to actual size!!

        let altered_frame_size = vec![
            0x28, 0xB5, 0x2F, 0xFD, 0x24, 0x28, 0x2D, 0x02, 0x00, 0x62, 0x45, 0x10, 0x11, 0xA0,
            0xED, 0x78, 0x78, 0x0F, 0xFA, 0x02, 0x02, 0x6D, 0x44, 0x74, 0xAB, 0x5E, 0x63, 0xBE,
            0x67, 0x46, 0x20, 0x84, 0x9D, 0xC9, 0x9D, 0x15, 0x3E, 0xA3, 0xEF, 0xF7, 0xB3, 0x4D,
            0x99, 0x6A, 0x73, 0x66, 0x55, 0xEE, 0xDD, 0xEF, 0x2E, 0x7D, 0x67, 0x72, 0x5F, 0xA5,
            0x0D, 0x5D, 0xAA, 0x8B, 0xE5, 0x84, 0xCE, 0x29, 0xEE, 0x97, 0x5E, 0xE9, 0x09, 0x08,
            0xE1, 0x70, 0xEB, 0xF2, 0x66, 0xDE, 0x11, 0x00, 0x20, 0x1F, 0x7A, 0x48,
        ];

        let result = StringPool::unpack(&altered_frame_size, 1, format, true);
        assert!(matches!(
            result,
            Err(StringPoolUnpackError::FailedToDecompress(NxDecompressionError::ZStandard(err)))
            if matches!(err,
                zstd_sys::ZSTD_ErrorCode::ZSTD_error_srcSize_wrong |
                zstd_sys::ZSTD_ErrorCode::ZSTD_error_corruption_detected
            )
        ));
    }

    #[rstest]
    #[case(StringPoolFormat::V0)]
    #[cfg_attr(miri, ignore)]
    fn unpack_fails_when_zstd_frame_size_made_too_large(#[case] format: StringPoolFormat) {
        // Compressed data with altered frame size 86 -> 120 bytes.
        // Data is following text:
        //
        //      Hello World!!
        //      I am an evil file, my frame size is too large compared to actual size!!

        let altered_frame_size = vec![
            0x28, 0xB5, 0x2F, 0xFD, 0x24, 0x78, 0x35, 0x02, 0x00, 0x62, 0x85, 0x10, 0x11, 0xA0,
            0xED, 0x78, 0x78, 0xE7, 0x1F, 0x07, 0x8B, 0x36, 0x22, 0xBA, 0x55, 0xAF, 0x31, 0xDF,
            0x33, 0x23, 0x20, 0x04, 0xAE, 0x0A, 0xBE, 0x57, 0x46, 0xB3, 0x6F, 0xF1, 0x87, 0xDF,
            0xBA, 0xD5, 0xCC, 0x39, 0xBD, 0xED, 0xB7, 0x16, 0x0F, 0xD5, 0xB9, 0x2A, 0x78, 0x5E,
            0xFB, 0xD0, 0xE9, 0xBA, 0xE0, 0x56, 0xE8, 0xAD, 0x26, 0x9F, 0xED, 0xD7, 0x9E, 0x80,
            0x10, 0x4E, 0xBF, 0x56, 0xDE, 0xBA, 0x79, 0x04, 0x00, 0xE2, 0xC7, 0x15, 0x50,
        ];

        let result = StringPool::unpack(&altered_frame_size, 1, format, true);
        assert!(matches!(
            result,
            Err(StringPoolUnpackError::FailedToDecompress(NxDecompressionError::ZStandard(err)))
            if matches!(err,
                zstd_sys::ZSTD_ErrorCode::ZSTD_error_srcSize_wrong |
                zstd_sys::ZSTD_ErrorCode::ZSTD_error_corruption_detected
            )
        ));
    }

    #[cfg(feature = "hardened")]
    #[rstest]
    #[case(StringPoolFormat::V0)]
    fn detect_overflow_when_file_count_too_small(#[case] format: StringPoolFormat) {
        let data = b"test1\0test2\0"; // 2 entries but expecting 3 entries
        let result = StringPool::unpack(data, 3, format, false);

        /*
            Note: This causes a buffer overflow because length of internal raw_data is derived from
            (decompressed_size - file_count). Essentially existing data, minus number of null terminators,
            which equals the file count.

            In this case, the code expects 3 null terminators, because file_count is 3, but only 2
            are present in the input. This means the `raw_data` allocation will be short by 1 byte.
        */

        assert!(matches!(result, Err(StringPoolUnpackError::BufferOverflow)));
    }

    #[rstest]
    #[case(StringPoolFormat::V0)]
    fn ignores_strings_beyond_expected_count(#[case] format: StringPoolFormat) {
        let data = b"test1\0test2\0test3\0test4\0"; // 4 entries but expecting 3 entries
        let result = StringPool::unpack(data, 3, format, false).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains("test1"));
        assert!(result.contains("test2"));
        assert!(result.contains("test3"));

        // Last entry is not present
        assert!(!result.contains("test4"));
    }

    #[cfg(feature = "hardened")]
    #[rstest]
    #[case(StringPoolFormat::V0)]
    fn parses_successfully_when_missing_final_null_terminator(#[case] format: StringPoolFormat) {
        let data = b"test1\0test2\0test3"; // missing final null terminator
        let result = StringPool::unpack(data, 3, format, false);

        // A final null terminator is missing.
        // This is technically not fatal, but would result in the final character being chopped off.
        assert!(matches!(
            result,
            Err(StringPoolUnpackError::ShouldEndOnNullTerminator)
        ));
    }
}
