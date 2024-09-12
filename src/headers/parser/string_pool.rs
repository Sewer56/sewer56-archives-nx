use super::string_pool_common::{
    self, StringPoolFormat, StringPoolPackError, StringPoolUnpackError,
};
use crate::utilities::compression::zstd::{
    self, compress_no_copy_fallback, max_alloc_for_compress_size,
};
use crate::{
    api::traits::has_relative_path::HasRelativePath,
    headers::raw::native_toc_header::NativeTocHeader,
};
use core::marker::PhantomData;
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
    pub fn pack<T: HasRelativePath>(
        items: &mut [T],
        format: StringPoolFormat,
    ) -> Result<Vec<u8>, StringPoolPackError> {
        match format {
            StringPoolFormat::V0 => Self::pack_v0_with_allocators(items, Global, Global),
            StringPoolFormat::V1 => Self::pack_v1_with_allocators(items, Global, Global),
        }
    }

    /// Unpacks a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `source` - The compressed data to unpack.
    /// * `file_count` - Number of files in the archive. This is equal to number of entries.
    /// * `format` - The (file) format of the string pool
    pub fn unpack(
        source: &[u8],
        file_count: usize,
        format: StringPoolFormat,
    ) -> Result<Self, StringPoolUnpackError> {
        match format {
            StringPoolFormat::V0 => {
                Self::unpack_v0_with_allocators(source, file_count, Global, Global)
            }
            StringPoolFormat::V1 => {
                Self::unpack_v1_with_allocators(source, file_count, Global, Global)
            }
        }
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

    /// Packs a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `items` - The list of items to pack
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    ///
    /// # Remarks
    ///
    /// For the file format details, see the [StringPoolFormat::V0] documentation.
    pub fn pack_v0_with_allocators<T: HasRelativePath>(
        items: &mut [T],
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
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
            decompressed_pool[offset + path_len].write(0);
            offset += path_len + 1;
        }

        let decompressed_pool = unsafe { decompressed_pool.assume_init() };
        Self::compress_pool(&decompressed_pool, long_alloc)
    }

    /// Unpacks a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `source` - The compressed data to unpack.
    /// * `file_count` - Number of files in the archive. This is equal to number of entries.
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    ///
    /// # Remarks
    ///
    /// For the file format details, see the [StringPoolFormat::V0] documentation.
    pub fn unpack_v0_with_allocators(
        source: &[u8],
        file_count: usize,
        _short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
    ) -> Result<StringPool<ShortAlloc, LongAlloc>, StringPoolUnpackError> {
        // Determine size of decompressed data
        // Note: This is very fast `O(1)` because the zstd frame header will contain the necessary info.
        let decompressed_size = zstd::get_decompressed_size(source)?;

        // Decompress the data
        let decompressed = Box::new_uninit_slice_in(decompressed_size, long_alloc.clone());
        let mut decompressed = unsafe { decompressed.assume_init() };
        zstd::decompress(source, &mut decompressed[..])?;
        // Populate all offsets
        let str_offsets: Box<[MaybeUninit<u32>], LongAlloc> =
            Box::new_uninit_slice_in(file_count, long_alloc.clone());
        let mut str_offsets = unsafe { str_offsets.assume_init() };

        // TODO: https://github.com/BurntSushi/memchr/issues/160
        // Add compile-time substitution.
        let mut memchr_iter = Memchr::new(0, &decompressed);
        let mut current_offset = 0;
        let mut offset_index = 0;

        while offset_index < file_count {
            str_offsets[offset_index] = current_offset;
            offset_index += 1;

            if let Some(null_pos) = memchr_iter.next() {
                current_offset = (null_pos as u32) + 1; // +1 to skip the null terminator
            } else {
                // If we've reached the end of the data, break the loop
                break;
            }
        }

        Ok(StringPool {
            _offsets: str_offsets,
            _raw_data: decompressed,
            _temp_allocator: PhantomData,
            _comp_allocator: PhantomData,
        })
    }

    /// Packs a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `items` - The list of items to pack
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    ///
    /// # Remarks
    ///
    /// For the file format details, see the [StringPoolFormat::V1] documentation.
    pub fn pack_v1_with_allocators<T: HasRelativePath>(
        items: &mut [T],
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
    ) -> Result<Vec<u8, LongAlloc>, StringPoolPackError> {
        items.sort_by(|a, b| a.relative_path().cmp(b.relative_path()));

        // Calculate total size: lengths + string data
        let pool_size = calc_pool_size(items);

        // Allocate uninitialized memory
        let mut decompressed_pool: Box<[MaybeUninit<u8>], ShortAlloc> =
            Box::new_uninit_slice_in(pool_size, short_alloc);

        // Write lengths to the start of the buffer.
        for idx in 0..items.len() {
            let length = items[idx].relative_path().len();
            if length > 255 {
                return Err(StringPoolPackError::FilePathTooLong);
            }
            decompressed_pool[idx].write(length as u8);
        }

        // Write string data
        let mut offset = items.len();
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
            offset += path_len;
        }

        let decompressed_pool = unsafe { decompressed_pool.assume_init() };
        Self::compress_pool(&decompressed_pool, long_alloc)
    }

    /// Unpacks a list of items into a string pool in its native binary format.
    /// For more details, read [`StringPool`].
    ///
    /// # Arguments
    /// * `source` - The compressed data to unpack.
    /// * `file_count` - Number of files in the archive. This is equal to number of entries.
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    ///
    /// # Remarks
    ///
    /// For the file format details, see the [StringPoolFormat::V1] documentation.
    pub fn unpack_v1_with_allocators(
        source: &[u8],
        file_count: usize,
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
    ) -> Result<StringPool<ShortAlloc, LongAlloc>, StringPoolUnpackError> {
        todo!("unimplemented");
        /*
        // Determine size of decompressed data
        let decompressed_size = zstd::get_decompressed_size(&source)?;

        // Allocate space for decompressed data using long_alloc
        let mut decompressed_data: Box<[MaybeUninit<u8>], LongAlloc> =
            Box::new_uninit_slice_in(decompressed_size, long_alloc.clone());
        let decompressed_slice = unsafe {
            std::slice::from_raw_parts_mut(
                decompressed_data.as_mut_ptr() as *mut u8,
                decompressed_size,
            )
        };

        // Decompress the data
        zstd::decompress(&source[..source.len() - file_count * 4], decompressed_slice)?;

        // Calculate offsets
        let mut offsets: Box<[MaybeUninit<u32>], LongAlloc> =
            Box::new_uninit_slice_in(file_count, long_alloc.clone());
        let offsets_ptr = offsets.as_mut_ptr() as *mut u32;
        let mut current_offset = file_count as u32;

        for i in 0..file_count {
            unsafe {
                *offsets_ptr.add(i) = current_offset;
                current_offset += decompressed_slice[i] as u32;
            }
        }

        // Extract string data (everything after the lengths)
        let string_data_size = decompressed_size - file_count;
        let mut string_data: Box<[u8], LongAlloc> =
            Box::new_uninit_slice_in(string_data_size, long_alloc);
        unsafe {
            copy_nonoverlapping(
                decompressed_slice.as_ptr().add(file_count),
                string_data.as_mut_ptr() as *mut u8,
                string_data_size,
            );
        }

        Ok(StringPool {
            _raw_data: unsafe { string_data.assume_init() },
            _offsets: unsafe { offsets.assume_init() },
            _temp_allocator: PhantomData,
            _comp_allocator: PhantomData,
        })
        */
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
                if destination.len() <= NativeTocHeader::MAX_STRING_POOL_SIZE {
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
    use std::alloc::System;

    use crate::{
        api::traits::has_relative_path::HasRelativePath,
        headers::parser::{
            string_pool::{StringPool, StringPoolUnpackError},
            string_pool_common::{StringPoolFormat::*, StringPoolPackError},
        },
    };

    #[derive(Debug, PartialEq, Eq)]
    struct TestItem {
        path: String,
    }

    impl HasRelativePath for TestItem {
        fn relative_path(&self) -> &str {
            &self.path
        }
    }

    #[test]
    fn can_pack_and_unpack() {
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

        let packed = StringPool::pack(&mut items, V0).unwrap();
        let unpacked = StringPool::unpack(&packed, items.len(), V0).unwrap();

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

    #[test]
    fn can_pack_empty_list() {
        let mut items: Vec<TestItem> = Vec::new();
        let packed = StringPool::pack(&mut items, V0).unwrap();
        assert!(!packed.is_empty()); // Even an empty pool should have some metadata

        let unpacked = StringPool::unpack(&packed, 0, V0).unwrap();
        assert_eq!(unpacked.len(), 0);
    }

    #[test]
    fn can_pack_large_list() {
        let mut items: Vec<TestItem> = (0..10000)
            .map(|i| TestItem {
                path: format!("file_{:05}.txt", i),
            })
            .collect();

        let packed = StringPool::pack(&mut items, V0).unwrap();
        let unpacked = StringPool::unpack(&packed, items.len(), V0).unwrap();

        assert_eq!(unpacked.len(), items.len());
        (0..unpacked.len())
            .for_each(|x| unsafe { assert_eq!(items[x].path, unpacked.get_unchecked(x)) });
    }

    #[test]
    fn unpack_invalid_data() {
        let invalid_data = vec![0, 1, 2, 3, 4]; // Invalid compressed data
        let result = StringPool::unpack(&invalid_data, 1, V0);
        assert!(matches!(
            result,
            Err(StringPoolUnpackError::FailedToGetDecompressedSize(_)
                | StringPoolUnpackError::FailedToDecompress(_))
        ));
    }

    #[test]
    fn pack_with_custom_allocators() {
        let mut items = vec![
            TestItem {
                path: "data/textures/cat.png".to_string(),
            },
            TestItem {
                path: "data/textures/dog.png".to_string(),
            },
        ];

        let packed = StringPool::pack_v0_with_allocators(&mut items, System, System).unwrap();
        let unpacked =
            StringPool::unpack_v0_with_allocators(&packed, items.len(), System, System).unwrap();

        assert_eq!(unpacked.len(), items.len());
        for item in &items {
            assert!(unpacked.contains(&item.path));
        }
    }

    #[test]
    fn v0_can_use_paths_over_256chars() {
        let mut items = vec![
            TestItem {
                // Exceeds 256 chars
                path: "/".to_owned() + &"a".repeat(255) + "/file.txt",
            },
            TestItem {
                path: "data/textures/cat.png".to_string(),
            },
        ];

        let packed = StringPool::pack(&mut items, V0).unwrap();
        let unpacked = StringPool::unpack(&packed, items.len(), V0).unwrap();

        assert_eq!(unpacked.len(), items.len());
        for item in &items {
            assert!(unpacked.contains(&item.path));
        }
    }

    /*
    #[test]
    fn v1_can_use_paths_up_to_255chars() {
        let mut items = vec![
            TestItem {
                // 255 chars
                path: "/".to_owned() + &"a".repeat(254),
            },
            TestItem {
                path: "data/textures/cat.png".to_string(),
            },
        ];

        let packed = StringPool::pack(&mut items, V1).unwrap();
        let unpacked = StringPool::unpack(&packed, items.len(), V1).unwrap();

        assert_eq!(unpacked.len(), items.len());
        for item in &items {
            assert!(unpacked.contains(&item.path));
        }
    }

    #[test]
    fn v1_cannot_use_paths_over_255chars() {
        let mut items = vec![
            TestItem {
                // 256 chars
                path: "/".to_owned() + &"a".repeat(255),
            },
            TestItem {
                path: "data/textures/cat.png".to_string(),
            },
        ];

        let result = StringPool::pack(&mut items, V1);
        assert!(matches!(result, Err(StringPoolPackError::FilePathTooLong)));
    }
    */

    #[test]
    fn can_use_non_ascii_paths() {
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

        let packed = StringPool::pack(&mut items, V0).unwrap();
        let unpacked = StringPool::unpack(&packed, items.len(), V0).unwrap();

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
}
