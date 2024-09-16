use crate::api::enums::compression_preference::CompressionPreference;
use crate::headers::enums::table_of_contents_version::TableOfContentsVersion;
use crate::headers::managed::block_size::BlockSize;
use crate::headers::managed::file_entry::FileEntry;
use crate::headers::parser::string_pool::StringPool;
use crate::headers::parser::string_pool_common::StringPoolUnpackError;
use crate::headers::raw::native_toc_block_entry::NativeTocBlockEntry;
use crate::headers::raw::native_toc_header::NativeTocHeader;
use crate::utilities::serialize::*;
use core::slice;
use little_endian_reader::LittleEndianReader;
use std::alloc::{Allocator, Global};

// TODO: Make this read only
/// Managed representation of the deserialized table of contents.
pub struct TableOfContents<
    ShortAlloc: Allocator + Clone = Global,
    LongAlloc: Allocator + Clone = Global,
> {
    /// Used formats for compression of each block.
    pub block_compressions: Box<[CompressionPreference], LongAlloc>,

    /// Individual block sizes in this structure.
    pub blocks: Box<[BlockSize], LongAlloc>,

    /// Individual file entries.
    pub entries: Box<[FileEntry], LongAlloc>,

    /// String pool data.
    pub pool: StringPool<ShortAlloc, LongAlloc>,

    /// Contains the version of the table of contents.
    pub version: TableOfContentsVersion,
}

impl TableOfContents {
    /// Deserializes the table of contents from a given address and version.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it works with raw pointers.
    ///
    /// # Arguments
    ///
    /// * `data_ptr` - Pointer to the ToC.
    /// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
    /// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
    ///
    /// # Returns
    ///
    /// Result containing the deserialized table of contents or a DeserializeError.
    pub unsafe fn deserialize(data_ptr: *const u8) -> Result<Self, DeserializeError> {
        Self::deserialize_with_allocator(data_ptr, Global, Global)
    }
}

impl<ShortAlloc, LongAlloc> TableOfContents<ShortAlloc, LongAlloc>
where
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
{
    /// Deserializes the table of contents from a given address and version.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it works with raw pointers.
    ///
    /// # Arguments
    ///
    /// * `data_ptr` - Pointer to the ToC.
    ///
    /// # Returns
    ///
    /// Result containing the deserialized table of contents or a DeserializeError.
    pub unsafe fn deserialize_with_allocator(
        data_ptr: *const u8,
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
    ) -> Result<Self, DeserializeError> {
        let mut reader = LittleEndianReader::new(data_ptr);
        let toc_header = NativeTocHeader::from_raw(reader.read::<u64>());

        let toc_version = match toc_header.get_version() {
            Ok(x) => x,
            Err(_) => return Err(DeserializeError::UnsupportedTocVersion),
        };

        // Init the vec and resize it to the correct length.
        let mut entries: Box<[FileEntry], LongAlloc> =
            Box::new_uninit_slice_in(toc_header.file_count() as usize, long_alloc.clone())
                .assume_init();
        let mut blocks: Box<[BlockSize], LongAlloc> =
            Box::new_uninit_slice_in(toc_header.block_count() as usize, long_alloc.clone())
                .assume_init();
        let mut block_compressions: Box<[CompressionPreference], LongAlloc> =
            Box::new_uninit_slice_in(toc_header.block_count() as usize, long_alloc.clone())
                .assume_init();

        // Read all of the ToC entries.
        if !entries.is_empty() {
            match toc_version {
                TableOfContentsVersion::V0 => {
                    for entry in &mut entries {
                        entry.from_reader_v0(&mut reader);
                    }
                }
                TableOfContentsVersion::V1 => {
                    for entry in &mut entries {
                        entry.from_reader_v1(&mut reader);
                    }
                }
            }
        }

        read_blocks_unrolled(&mut blocks, &mut block_compressions, &mut reader);

        let pool = StringPool::unpack_v0_with_allocators(
            slice::from_raw_parts(reader.ptr, toc_header.string_pool_size() as usize),
            toc_header.file_count() as usize,
            short_alloc.clone(),
            long_alloc.clone(),
        )
        .map_err(DeserializeError::StringPoolUnpackError)?;

        Ok(TableOfContents {
            block_compressions,
            blocks,
            entries,
            pool,
            version: toc_version,
        })
    }
}

/// Helper function to read blocks in an unrolled manner for performance.
pub fn read_blocks_unrolled(
    blocks: &mut [BlockSize],
    compressions: &mut [CompressionPreference],
    reader: &mut LittleEndianReader,
) {
    let blocks_len = blocks.len();
    let blocks_ptr = blocks.as_mut_ptr();
    let compressions_ptr = compressions.as_mut_ptr();

    // SAFETY: We're just avoiding bounds checks here, we know that blocks_len == compressions_len
    //         by definition, so there is no risk of overflowing.
    unsafe {
        for x in 0..blocks_len {
            let value = NativeTocBlockEntry::from_reader(reader);
            *blocks_ptr.add(x) = BlockSize::new(value.compressed_block_size());
            *compressions_ptr.add(x) = value.compression();
        }
    }

    // Unrolled Version
    /*
        unsafe {
        let mut x = 0;
        while x + 4 <= blocks_len {
            let value1 = NativeTocBlockEntry::from_reader(reader);
            let value2 = NativeTocBlockEntry::from_reader(reader);
            let value3 = NativeTocBlockEntry::from_reader(reader);
            let value4 = NativeTocBlockEntry::from_reader(reader);

            *blocks_ptr.add(x) = BlockSize::new(value1.compressed_block_size());
            *blocks_ptr.add(x + 1) = BlockSize::new(value2.compressed_block_size());
            *blocks_ptr.add(x + 2) = BlockSize::new(value3.compressed_block_size());
            *blocks_ptr.add(x + 3) = BlockSize::new(value4.compressed_block_size());

            *compressions_ptr.add(x) = value1.compression();
            *compressions_ptr.add(x + 1) = value2.compression();
            *compressions_ptr.add(x + 2) = value3.compression();
            *compressions_ptr.add(x + 3) = value4.compression();

            x += 4;
        }

        // Handle remaining elements
        while x < blocks_len {
            let value = NativeTocBlockEntry::from_reader(reader);
            *blocks_ptr.add(x) = BlockSize::new(value.compressed_block_size());
            *compressions_ptr.add(x) = value.compression();
            x += 1;
        }
    }
    */
}

/// Errors that can occur when deserializing TableOfContents
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeserializeError {
    /// Error unpacking the string pool
    StringPoolUnpackError(StringPoolUnpackError),
    /// Unsupported table of contents version
    UnsupportedTocVersion,
}
