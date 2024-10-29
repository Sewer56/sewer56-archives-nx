use endian_writer::{EndianReader, EndianReaderExt, LittleEndianReader};

use crate::{
    api::enums::compression_preference::CompressionPreference,
    headers::{managed::*, parser::*, raw::toc::*},
};
use core::{hint::unreachable_unchecked, slice};
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
    pub unsafe fn deserialize_v2xx(data_ptr: *const u8) -> Result<Self, DeserializeError> {
        Self::deserialize_v2xx_with_allocator(data_ptr, Global, Global)
    }
}

impl<ShortAlloc, LongAlloc> TableOfContents<ShortAlloc, LongAlloc>
where
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
{
    /// Deserializes the table of contents [NX v2.x.x format] from a given address and version.
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
    /// Result containing the deserialized table of contents or a [`DeserializeError`].
    pub unsafe fn deserialize_v2xx_with_allocator(
        data_ptr: *const u8,
        short_alloc: ShortAlloc,
        long_alloc: LongAlloc,
    ) -> Result<Self, DeserializeError> {
        let mut reader = LittleEndianReader::new(data_ptr);

        // The first bit in all V2 header formats is the FEF flag.
        let toc_header = Preset3TocHeader::from_raw(reader.read_u64());

        if toc_header.get_is_flexible_format() {
            let toc_header = Fef64TocHeader::from_raw(toc_header.0);
            return deserialize_v2xx_fef64_entries(
                &mut reader,
                toc_header,
                short_alloc,
                long_alloc,
            );
        }

        // The next two bits are the preset (in all presets), and in Preset3 specifically,
        // the next bit is the hash flag. Therefore we can use Preset3 header to
        // read all of the necessary information here.
        let preset = toc_header.get_preset();
        if preset == 0 || preset == 1 || preset == 2 {
            let toc_header = Preset0TocHeader::from_raw(toc_header.0);
            deserialize_v2xx_preset_entries(
                &mut reader,
                toc_header.string_pool_size(),
                toc_header.block_count(),
                toc_header.file_count(),
                preset,
                true,
                short_alloc,
                long_alloc,
            )
        } else if preset == 3 {
            let toc_header = Preset3TocHeader::from_raw(toc_header.0);
            deserialize_v2xx_preset_entries(
                &mut reader,
                toc_header.string_pool_size(),
                toc_header.block_count() as u32,
                toc_header.file_count() as u32,
                preset,
                toc_header.has_hash(),
                short_alloc,
                long_alloc,
            )
        } else {
            // Unreachable by definition, since the preset_no is restricted to 2 bits.
            unreachable_unchecked()
        }
    }
}

/// Deserializes the file entries [NX v2.x.x format] which uses a preset
/// [as opposed to the flexible format].
///
/// # Safety
///
/// This function is unsafe because it works with raw pointers.
///
/// # Arguments
///
/// * `reader` - Allows for reading table of contents.
/// * `pool_size` - Size of the compressed string pool (bytes).
/// * `block_count` - Number of blocks in the table of contents.
/// * `file_count` - Number of files in the table of contents.
/// * `preset` - Preset number.
/// * `has_hash` - Whether the preset variant of table of contents has a hash.
///                [Applies only to variants where hash is optional]
/// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
/// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
///
/// # Returns
///
/// Result containing the deserialized table of contents or a [`DeserializeError`].
#[allow(clippy::too_many_arguments)]
unsafe fn deserialize_v2xx_preset_entries<ShortAlloc, LongAlloc>(
    reader: &mut LittleEndianReader,
    pool_size: u32,
    block_count: u32,
    file_count: u32,
    preset: u8,
    has_hash: bool,
    short_alloc: ShortAlloc,
    long_alloc: LongAlloc,
) -> Result<TableOfContents<ShortAlloc, LongAlloc>, DeserializeError>
where
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
{
    // Read the entries.
    let mut entries: Box<[FileEntry], LongAlloc> =
        Box::new_uninit_slice_in(file_count as usize, long_alloc.clone()).assume_init();

    if preset == 0 {
        reader.read_entries_into_unroll_2::<FileEntry, NativeFileEntryP0>(&mut entries);
    } else if preset == 1 {
        reader.read_entries_into_unroll_2::<FileEntry, NativeFileEntryP1>(&mut entries);
    } else if preset == 2 {
        reader.read_entries_into_unroll_2::<FileEntry, NativeFileEntryP2>(&mut entries);
    } else if preset == 3 {
        if has_hash {
            reader.read_entries_into_unroll_2::<FileEntry, NativeFileEntryP3>(&mut entries);
        } else {
            reader.read_entries_into_unroll_2::<FileEntry, NativeFileEntryP3NoHash>(&mut entries);
        }
    }

    read_stuff_after_entries_and_return_toc(
        reader,
        entries,
        block_count,
        pool_size,
        file_count,
        long_alloc,
        short_alloc,
    )
}

/// Deserializes the file entries [NX v2.x.x format] which uses the Flexible Format
/// [as opposed to a Preset].
///
/// # Safety
///
/// This function is unsafe because it works with raw pointers.
///
/// # Arguments
///
/// * `reader` - Allows for reading table of contents. Should be seeked just past the 8 byte header.
/// * `toc_header` - 8 byte table of contents header for flexible format.
/// * `short_alloc` - Allocator for short lived memory. Think pooled memory and rentals.
/// * `long_alloc` - Allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
///
/// # Returns
///
/// Result containing the deserialized table of contents or a [`DeserializeError`].
unsafe fn deserialize_v2xx_fef64_entries<ShortAlloc, LongAlloc>(
    reader: &mut LittleEndianReader,
    toc_header: Fef64TocHeader,
    short_alloc: ShortAlloc,
    long_alloc: LongAlloc,
) -> Result<TableOfContents<ShortAlloc, LongAlloc>, DeserializeError>
where
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
{
    let is_extended = toc_header.has_extended_header();
    let fields_bytes: FileEntryFieldsBits = toc_header.into();
    let counts_raw_bytes = if is_extended {
        toc_header.padding_or_item_counts()
    } else {
        reader.read_u64()
    };
    let (pool_size, block_count, file_count) = unpack_item_counts(
        counts_raw_bytes,
        toc_header.string_pool_size_bits(),
        toc_header.block_count_bits(),
        toc_header.file_count_bits(),
    );

    // Read the entries.
    let mut entries: Box<[FileEntry], LongAlloc> =
        Box::new_uninit_slice_in(file_count as usize, long_alloc.clone()).assume_init();

    let mut file_entry_ptr = entries.as_mut_ptr();
    if toc_header.has_hash() {
        for _idx in 0..file_count {
            let entry16 = FileEntry16::from_reader(reader);
            *file_entry_ptr = entry16.to_file_entry(fields_bytes);
            file_entry_ptr = file_entry_ptr.add(1);
        }
    } else {
        for _idx in 0..file_count {
            let entry8 = FileEntry8::from_reader(reader);
            *file_entry_ptr = entry8.to_file_entry(fields_bytes);
            file_entry_ptr = file_entry_ptr.add(1);
        }
    }

    read_stuff_after_entries_and_return_toc(
        reader,
        entries,
        block_count as u32,
        pool_size as u32,
        file_count as u32,
        long_alloc,
        short_alloc,
    )
}

unsafe fn read_stuff_after_entries_and_return_toc<ShortAlloc, LongAlloc>(
    reader: &mut LittleEndianReader,
    entries: Box<[FileEntry], LongAlloc>,
    block_count: u32,
    pool_size: u32,
    file_count: u32,
    long_alloc: LongAlloc,
    short_alloc: ShortAlloc,
) -> Result<TableOfContents<ShortAlloc, LongAlloc>, DeserializeError>
where
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
{
    // Read blocks after files
    let mut block_compressions: Box<[CompressionPreference], LongAlloc> =
        Box::new_uninit_slice_in(block_count as usize, long_alloc.clone()).assume_init();
    let mut blocks: Box<[BlockSize], LongAlloc> =
        Box::new_uninit_slice_in(block_count as usize, long_alloc.clone()).assume_init();
    read_blocks_unrolled(&mut blocks, &mut block_compressions, reader);

    // Read the pool and return.
    let pool = StringPool::unpack_v0_with_allocators(
        slice::from_raw_parts(reader.ptr, pool_size as usize),
        file_count as usize,
        short_alloc.clone(),
        long_alloc.clone(),
        true,
    )?;

    Ok(TableOfContents {
        block_compressions,
        blocks,
        entries,
        pool,
    })
}

/// Helper function to read blocks in an unrolled manner for performance.
fn read_blocks_unrolled(
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
            let value1 = NativeV2TocBlockEntry::from_reader(lereader);
            let value2 = NativeV2TocBlockEntry::from_reader(lereader);
            let value3 = NativeV2TocBlockEntry::from_reader(lereader);
            let value4 = NativeV2TocBlockEntry::from_reader(lereader);

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
            let value = NativeV2TocBlockEntry::from_reader(lereader);
            *blocks_ptr.add(x) = BlockSize::new(value.compressed_block_size());
            *compressions_ptr.add(x) = value.compression();
            x += 1;
        }
    }
}
