use endian_writer::{EndianReader, LittleEndianReader};

use crate::{
    api::enums::compression_preference::CompressionPreference,
    headers::{enums::v1::*, managed::*, parser::*, raw::toc::*},
};
use core::slice;
use std::alloc::{Allocator, Global};

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
    pub unsafe fn deserialize_v1xx(data_ptr: *const u8) -> Result<Self, DeserializeError> {
        Self::deserialize_v1xx_with_allocator(data_ptr, Global, Global)
    }
}

impl<ShortAlloc, LongAlloc> TableOfContents<ShortAlloc, LongAlloc>
where
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
{
    /// Deserializes the table of contents [NX v1.x.x format] from a given address and version.
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
    pub unsafe fn deserialize_v1xx_with_allocator(
        data_ptr: *const u8,
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
    ) -> Result<Self, DeserializeError> {
        // TODO: 'harden' this code against out of bounds reads.
        let mut reader = LittleEndianReader::new(data_ptr);
        let toc_header = NativeTocHeader::from_raw(reader.read_u64());

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
        // Perf: Nothing gained here from unrolling.
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
            true,
        )
        .map_err(DeserializeError::StringPoolUnpackError)?;

        Ok(TableOfContents {
            block_compressions,
            blocks,
            entries,
            pool,
        })
    }
}

/// Helper function to read blocks in an unrolled manner for performance.
pub fn read_blocks_unrolled(
    blocks: &mut [BlockSize],
    compressions: &mut [CompressionPreference],
    lereader: &mut LittleEndianReader,
) {
    let blocks_len = blocks.len();
    let blocks_ptr = blocks.as_mut_ptr();
    let compressions_ptr = compressions.as_mut_ptr();

    // SAFETY: We're just avoiding bounds checks here, we know that blocks_len == compressions_len
    //         by definition, so there is no risk of overflowing.

    // Unrolled Version
    unsafe {
        let mut x = 0;
        while x + 4 <= blocks_len {
            let value1 = NativeV1TocBlockEntry::from_reader(lereader);
            let value2 = NativeV1TocBlockEntry::from_reader(lereader);
            let value3 = NativeV1TocBlockEntry::from_reader(lereader);
            let value4 = NativeV1TocBlockEntry::from_reader(lereader);

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
            let value = NativeV1TocBlockEntry::from_reader(lereader);
            *blocks_ptr.add(x) = BlockSize::new(value.compressed_block_size());
            *compressions_ptr.add(x) = value.compression();
            x += 1;
        }
    }
}
