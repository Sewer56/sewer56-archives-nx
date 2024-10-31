use crate::{
    api::{enums::compression_preference::CompressionPreference, traits::*},
    headers::{managed::*, parser::*, raw::toc::*},
    implementation::pack::{
        blocks::polyfills::Block, table_of_contents_builder_state::TableOfContentsBuilderState,
    },
};
use core::hint::unreachable_unchecked;
use endian_writer::{EndianWriter, EndianWriterExt, LittleEndianWriter};
use nanokit::count_bits::BitsNeeded;
use std::alloc::Allocator;
use thiserror_no_std::Error;

/// Holds the result of initializing the creation of a binary Table of Contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderInfo<LongAlloc: Allocator + Clone> {
    /// The format the table of contents should be written in.
    pub format: ToCFormat,
    /// Indicates whether any block can create chunks.
    /// This affects the packer behaviour, with regards to
    /// how much 'working' memory we need to allocate.
    pub can_create_chunks: bool,
    /// The size of the table.
    pub table_size: u32,
    /// Maximum offset of decompressed data in a block.
    pub max_decomp_block_offset: u32,
    /// The raw data for the StringPool.
    pub string_pool: Vec<u8, LongAlloc>,
}

/// Determines the required Table of Contents version based on the files present within the given
/// set of blocks.
///
/// # Arguments
///
/// * `blocks` - A slice of [Box<dyn Block<T>>] representing the blocks in the archive.
/// * `chunk_size` - The maximum size of a chunk in the file. From [PackingSettings].
/// * `max_block_size` - The maximum size of a SOLID block. From [PackingSettings].
/// * `need_hashes` - Whether to include hashes in the table of contents.
/// * `short_alloc` - An allocator for short lived memory. Think pooled memory and rentals.
/// * `long_alloc` - An allocator for longer lived memory. Think same lifetime as creating Nx archive creator/unpacker.
///
/// # Returns
///
/// A [BuilderInfo] struct containing the determined [ToCFormat] and a boolean
/// indicating if any block can create chunks.
///
/// # Type Parameters
///
/// * `T`: Type of the items in the blocks, which must implement [HasFileSize],
///        [CanProvideFileData], and [HasRelativePath].
pub fn init_toc_creation<
    T: HasFileSize + CanProvideFileData + HasRelativePath,
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
>(
    blocks: &[Box<dyn Block<T>>],
    chunk_size: u32,
    mut max_decomp_block_size: u32,
    need_hashes: bool,
    short_alloc: ShortAlloc,
    long_alloc: LongAlloc,
) -> Result<BuilderInfo<LongAlloc>, InitError> {
    let mut largest_file_size: u64 = 0;
    let mut can_create_chunks = false;
    let mut files = Vec::new();

    // Gather all files from blocks.
    for block in blocks {
        block.append_items(&mut files);

        let max_decomp_block_offset = block.max_decompressed_block_offset();
        if max_decomp_block_offset > max_decomp_block_size {
            max_decomp_block_size = max_decomp_block_offset;
        }
    }

    // Determine largest size, and whether we need to chunk.
    for file in &files {
        let file_size = file.file_size();
        if file_size > largest_file_size {
            largest_file_size = file_size;
        }

        if file_size > chunk_size as u64 {
            can_create_chunks = true;
        }
    }

    // Generate string pool
    let string_pool =
        StringPool::pack_v0_with_allocators(&mut files, short_alloc, long_alloc, true)?;

    // Determine table of contents format
    let max_block_ofs = max_decomp_block_size;
    let string_pool_len = string_pool.len() as u32;
    let block_count = blocks.len() as u32;
    let file_count = files.len() as u32;
    let format = determine_optimal_toc_format(
        string_pool_len,
        max_block_ofs,
        block_count,
        file_count,
        need_hashes,
        largest_file_size,
    );

    // Return error if the format is invalid.
    if format == ToCFormat::Error {
        return Err(InitError::NoSuitableTocFormat(format));
    }

    // Calculate table size.
    let toc_size = calculate_toc_size(format, string_pool_len, block_count, file_count);

    Ok(BuilderInfo {
        format,
        can_create_chunks,
        table_size: toc_size,
        max_decomp_block_offset: max_decomp_block_size,
        string_pool,
    })
}

/// Serializes the table of contents data into a binary format from a builder state.
///
/// This function takes the builder state of a table of contents and serializes it into a binary
/// format at the specified memory location.
///
/// # Safety
///
/// This function is unsafe because it writes to a raw pointer. The caller must ensure that:
/// - `data_ptr` points to a memory region large enough to hold the serialized data.
/// - The memory region pointed to by `data_ptr` is writable.
/// - The lifetime of the pointed memory is at least as long as the execution of this function.
///
/// # Arguments
///
/// * `builder_state` - Builder state constructed by the packing operation.
/// * `info` - The builder information constructed by init-ing the serialize operation with [init_toc_creation].
/// * `data_ptr` - A raw pointer to the memory where the data will be written.
///
/// # Returns
///
/// Returns a `Result` which is:
/// - `Ok(usize)` containing the number of bytes written if serialization is successful.
/// - `Err(SerializeError)` if an error occurs during serialization.
///
/// # Remarks
///
/// Before calling this, first call [init_toc_creation], to construct a StringPool, and related information.
/// With this you can call [calculate_table_size] to determine the table size; reserve the space; and then
/// pack the data; creating the [TableOfContentsBuilderState] to call this function with.
///
/// # Errors
///
/// This function will return an error if it is not possible to serialize the table of contents.
#[allow(dead_code)] // TODO: This is temporary
pub(crate) unsafe fn serialize_table_of_contents_from_state<LongAlloc: Allocator + Clone>(
    builder_state: &TableOfContentsBuilderState,
    info: &BuilderInfo<LongAlloc>,
    data_ptr: *mut u8,
) -> Result<usize, SerializeError> {
    serialize_table_of_contents(
        &builder_state.block_compressions,
        &builder_state.blocks,
        &builder_state.entries,
        info,
        data_ptr,
    )
}

/// Serializes the table of contents data into a binary format from a builder state.
///
/// This function takes the builder state of a table of contents and serializes it into a binary
/// format at the specified memory location.
///
/// # Safety
///
/// This function is unsafe because it writes to a raw pointer. The caller must ensure that:
/// - `data_ptr` points to a memory region large enough to hold the serialized data.
/// - The memory region pointed to by `data_ptr` is writable.
/// - The lifetime of the pointed memory is at least as long as the execution of this function.
///
/// # Arguments
///
/// * `block_compressions` - A slice of `CompressionPreference` values for each block.
/// * `blocks` - A slice of `BlockSize` structs representing the size of each block.
/// * `entries` - A slice of `FileEntry` structs representing file entries in the table.
/// * `info` - The builder information constructed by init-ing the serialize operation with [init_toc_creation].
/// * `data_ptr` - A raw pointer to the memory where the data will be written.
///
/// # Returns
///
/// Returns a `Result` which is:
/// - `Ok(usize)` containing the number of bytes written if serialization is successful.
/// - `Err(SerializeError)` if an error occurs during serialization.
///
/// # Errors
///
/// This function will return an error if it is not possible to serialize the table of contents.
pub unsafe fn serialize_table_of_contents<LongAlloc: Allocator + Clone>(
    block_compressions: &[CompressionPreference],
    blocks: &[BlockSize],
    entries: &[FileEntry],
    info: &BuilderInfo<LongAlloc>,
    data_ptr: *mut u8,
) -> Result<usize, SerializeError> {
    match info.format {
        ToCFormat::FEF64 => Ok(serialize_table_of_contents_fef64(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            true,
        )),
        ToCFormat::FEF64NoHash => Ok(serialize_table_of_contents_fef64(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            false,
        )),
        ToCFormat::Preset0 => Ok(serialize_table_of_contents_preset(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            0,
            false,
        )),
        ToCFormat::Preset1NoHash => Ok(serialize_table_of_contents_preset(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            1,
            false,
        )),
        ToCFormat::Preset2 => Ok(serialize_table_of_contents_preset(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            2,
            false,
        )),
        ToCFormat::Preset3 => Ok(serialize_table_of_contents_preset(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            3,
            true,
        )),
        ToCFormat::Preset3NoHash => Ok(serialize_table_of_contents_preset(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            3,
            false,
        )),
        ToCFormat::Error => Err(SerializeError::UnsupportedTocFormat),
    }
}

unsafe fn serialize_table_of_contents_fef64<LongAlloc: Allocator + Clone>(
    block_compressions: &[CompressionPreference],
    blocks: &[BlockSize],
    entries: &[FileEntry],
    info: &BuilderInfo<LongAlloc>,
    data_ptr: *mut u8,
    include_hash: bool,
) -> usize {
    let mut lewriter = LittleEndianWriter::new(data_ptr);

    // Write the ToC Header.
    let file_count_bits = entries.len().bits_needed_to_store() as u8;
    let block_count_bits = blocks.len().bits_needed_to_store() as u8;
    let string_pool_size_bits = info.string_pool.len().bits_needed_to_store() as u8;
    let block_offset_bits = info.max_decomp_block_offset.bits_needed_to_store() as u8;
    let needs_extra_8_bytes =
        !can_fit_within_42_bits(string_pool_size_bits, block_count_bits, file_count_bits);

    let mut item_counts: u64 = 0;
    pack_item_counts(
        info.string_pool.len() as u64,
        string_pool_size_bits,
        blocks.len() as u64,
        block_count_bits,
        entries.len() as u64,
        file_count_bits,
        &mut item_counts,
    );

    let header = Fef64TocHeader::new(
        include_hash,
        string_pool_size_bits,
        file_count_bits,
        block_count_bits,
        block_offset_bits,
        item_counts,
    );
    lewriter.write_u64(header.0);

    // Write extended header if needed
    if needs_extra_8_bytes {
        lewriter.write_u64(item_counts);
    }

    // Serialize the entries.
    let fields_bits =
        FileEntryFieldsBits::new(block_count_bits, file_count_bits, block_offset_bits);
    if include_hash {
        write_file_entries_with_hash(entries, fields_bits, &mut lewriter);
    } else {
        write_file_entries_without_hash(entries, fields_bits, &mut lewriter);
    }

    // Now write the blocks after the headers.
    if !blocks.is_empty() {
        write_blocks(blocks, block_compressions, &mut lewriter);
    }

    // Write the raw string pool data.
    lewriter.write_bytes(&info.string_pool);
    lewriter.ptr as usize - data_ptr as usize
}

#[inline(never)] // better register allocation since FileEntryFieldsBits uses a lot of regs
fn write_file_entries_without_hash(
    entries: &[FileEntry],
    fields_bits: FileEntryFieldsBits,
    lewriter: &mut LittleEndianWriter,
) {
    for item in entries {
        let entry8 = FileEntry8::from_file_entry(fields_bits, item);
        entry8.to_writer(lewriter);
    }
}

#[inline(never)] // better register allocation since FileEntryFieldsBits uses a lot of regs
fn write_file_entries_with_hash(
    entries: &[FileEntry],
    fields_bits: FileEntryFieldsBits,
    lewriter: &mut LittleEndianWriter,
) {
    for item in entries {
        let entry16 = FileEntry16::from_file_entry(fields_bits, item);
        entry16.to_writer(lewriter);
    }
}

unsafe fn serialize_table_of_contents_preset<LongAlloc: Allocator + Clone>(
    block_compressions: &[CompressionPreference],
    blocks: &[BlockSize],
    entries: &[FileEntry],
    info: &BuilderInfo<LongAlloc>,
    data_ptr: *mut u8,
    preset_no: u8,
    include_hash: bool,
) -> usize {
    let mut lewriter = LittleEndianWriter::new(data_ptr);

    // Write the initial header.
    if preset_no != 3 {
        let header = Preset0TocHeader::new(
            preset_no,
            info.string_pool.len() as u32,
            blocks.len() as u32,
            entries.len() as u32,
        );
        lewriter.write_u64(header.0);
    } else {
        let header = Preset3TocHeader::new(
            include_hash,
            info.string_pool.len() as u32,
            blocks.len() as u16,
            entries.len() as u16,
        );
        lewriter.write_u64(header.0);
    }

    // Serialize the entries.
    if preset_no == 0 {
        lewriter.write_entries_into_unroll_2::<NativeFileEntryP0, FileEntry>(entries);
    } else if preset_no == 1 {
        lewriter.write_entries_into_unroll_2::<NativeFileEntryP1, FileEntry>(entries);
    } else if preset_no == 2 {
        lewriter.write_entries_into_unroll_2::<NativeFileEntryP2, FileEntry>(entries);
    } else if preset_no == 3 {
        if include_hash {
            lewriter.write_entries_into_unroll_2::<NativeFileEntryP3, FileEntry>(entries);
        } else {
            lewriter.write_entries_into_unroll_2::<NativeFileEntryP3NoHash, FileEntry>(entries);
        }
    } else {
        // Unreachable by definition, since the preset_no is restricted to 2 bits.
        unreachable_unchecked();
    }

    // Now write the blocks after the headers.
    if !blocks.is_empty() {
        write_blocks(blocks, block_compressions, &mut lewriter);
    }

    // Write the raw string pool data.
    lewriter.write_bytes(&info.string_pool);
    lewriter.ptr as usize - data_ptr as usize
}

/// Errors that can occur when writing the binary table of contents.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum SerializeError {
    /// The format of the table of contents used is not supported.
    UnsupportedTocFormat,
}

/// Errors that can occur when initializing the creation of Table of Contents.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum InitError {
    /// There is not a ToC format that can accomodate for the requirements of the files
    /// that need to be archived.
    NoSuitableTocFormat(#[from] ToCFormat),

    /// Unsupported table of contents version
    FailedToCreateStringPool(#[from] StringPoolPackError),
}

/// Helper function to write blocks.
///
/// # Remarks
///
/// May unroll this manually depending on future benchmark results.
/// Doing write using pure pointer arithmetic and comparing with max address was not faster.
fn write_blocks(
    blocks: &[BlockSize],
    compressions: &[CompressionPreference],
    lewriter: &mut LittleEndianWriter,
) {
    // This makes the bounds checker leave us alone.
    debug_assert!(blocks.len() == compressions.len());

    // SAFETY: Debug&Test Builds Verify that both arrays have the same length
    //         They should have the same length by definition
    unsafe {
        for x in 0..blocks.len() {
            let num_blocks = (*blocks.as_ptr().add(x)).compressed_size;
            let compression = *compressions.as_ptr().add(x);
            NativeV2TocBlockEntry::to_writer(num_blocks, compression, lewriter);
        }
    }

    // No benefit found from unrolling here.
}

/// Calculates the size of the table after serialization to binary.
/// This is used for pre-allocating space needed for the table.
///
/// # Arguments
///
/// * `format` - The format the table will be serialized in.
/// * `string_pool_len` - Length of the serialized string pool.
/// * `block_count` - Number of blocks written to the file.
/// * `file_count` - Number of files written to the header.
///
/// # Returns
///
/// Size of the Table of Contents
pub fn calculate_toc_size(
    format: ToCFormat,
    string_pool_len: u32,
    block_count: u32,
    file_count: u32,
) -> u32 {
    let mut toc_size = 8; // size of all headers
    if (format == ToCFormat::FEF64 || format == ToCFormat::FEF64NoHash)
        && fef64_needs_extra_8bytes(string_pool_len, block_count, file_count)
    {
        toc_size += 8;
    }

    let entry_size = match format {
        ToCFormat::Preset3NoHash => size_of::<NativeFileEntryP3NoHash>() as u32,
        ToCFormat::FEF64NoHash => size_of::<FileEntry8>() as u32,
        ToCFormat::Preset1NoHash => size_of::<NativeFileEntryP1>() as u32,
        ToCFormat::Preset3 => size_of::<NativeFileEntryP3>() as u32,
        ToCFormat::FEF64 => size_of::<FileEntry16>() as u32,
        ToCFormat::Preset0 => size_of::<NativeFileEntryP0>() as u32,
        ToCFormat::Preset2 => size_of::<NativeFileEntryP2>() as u32,
        ToCFormat::Error => 0,
    };

    toc_size += entry_size * file_count;
    toc_size += block_count * size_of::<NativeV2TocBlockEntry>() as u32;
    toc_size += string_pool_len;
    toc_size
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::alloc::Global;

    // Shared test setup
    fn create_test_data(
        format: ToCFormat,
    ) -> (TableOfContentsBuilderState<'static>, BuilderInfo<Global>) {
        // Generate dummy data for archived file
        let mut toc_builder = unsafe { TableOfContentsBuilderState::new(4, 5) };

        let entries = [
            FileEntry::new(0xBBBBBBBBBBBBBBBB, 16_777_215, 0, 0, 0), // Chunked file (2 chunks)
            FileEntry::new(0xCCCCCCCCCCCCCCCC, 8_388_607, 0, 1, 2),  // Single chunk file
            FileEntry::new(0xDDDDDDDDDDDDDDDD, 256, 0, 2, 3),        // 3rd block, SOLID
            FileEntry::new(0xEEEEEEEEEEEEEEEE, 512, 256, 3, 3),      // SOLID block file 2
            FileEntry::new(0xFFFFFFFFFFFFFFFF, 1024, 768, 4, 3), // SOLID block file 3. Max offset: 1792
        ];

        let blocks = [
            BlockSize::new(8_388_608), // File 1
            BlockSize::new(8_388_607),
            BlockSize::new(8_388_607), // File 2
            BlockSize::new(1792),      // SOLID Block
        ];

        let block_compressions = [
            CompressionPreference::Copy,
            CompressionPreference::ZStandard,
            CompressionPreference::Lz4,
            CompressionPreference::Copy,
        ];

        // Emulate a write to entries by the packing operation
        for (x, entry) in entries.iter().enumerate() {
            toc_builder.set_entry(x, *entry).unwrap();
        }

        for (x, block) in blocks.iter().enumerate() {
            toc_builder.set_block(x, *block).unwrap();
        }

        for (x, compression) in block_compressions.iter().enumerate() {
            toc_builder.set_block_compression(x, *compression).unwrap();
        }

        // Create builder info
        let builder_info = BuilderInfo {
            format,
            can_create_chunks: true,
            table_size: calculate_toc_size(format, 0, blocks.len() as u32, entries.len() as u32),
            max_decomp_block_offset: 4096,
            string_pool: Vec::new(), // Empty string pool for test
        };

        (toc_builder, builder_info)
    }

    fn serialize_test_data(
        format: ToCFormat,
    ) -> (
        Vec<u8>,
        TableOfContentsBuilderState<'static>,
        BuilderInfo<Global>,
    ) {
        let (toc_builder, builder_info) = create_test_data(format);

        let mut data = vec![0u8; builder_info.table_size as usize];
        unsafe {
            let bytes_written = serialize_table_of_contents_from_state(
                &toc_builder,
                &builder_info,
                data.as_mut_ptr(),
            )
            .unwrap();
            assert_eq!(bytes_written, builder_info.table_size as usize);
        }

        (data, toc_builder, builder_info)
    }

    fn assert_file_entries_equal(
        original: &FileEntry,
        deserialized: &FileEntry,
        index: usize,
        has_hash: bool,
        has_block_offset: bool,
    ) {
        if has_hash {
            assert_eq!(
                original.hash, deserialized.hash,
                "Mismatch in hash for entry {}",
                index
            );
        }

        assert_eq!(
            original.decompressed_size, deserialized.decompressed_size,
            "Mismatch in decompressed_size for entry {}",
            index
        );

        if has_block_offset {
            assert_eq!(
                original.decompressed_block_offset, deserialized.decompressed_block_offset,
                "Mismatch in decompressed_block_offset for entry {}",
                index
            );
        }

        assert_eq!(
            original.file_path_index, deserialized.file_path_index,
            "Mismatch in file_path_index for entry {}",
            index
        );
        assert_eq!(
            original.first_block_index, deserialized.first_block_index,
            "Mismatch in first_block_index for entry {}",
            index
        );
    }

    #[rstest]
    #[case::preset0(ToCFormat::Preset0)]
    #[case::preset1_no_hash(ToCFormat::Preset1NoHash)]
    #[case::preset2(ToCFormat::Preset2)]
    #[case::preset3(ToCFormat::Preset3)]
    #[case::preset3_no_hash(ToCFormat::Preset3NoHash)]
    #[case::fef64(ToCFormat::FEF64)]
    #[case::fef64_no_hash(ToCFormat::FEF64NoHash)]
    fn can_serialize_and_deserialize(#[case] format: ToCFormat) {
        // Create test data with the specified format
        let (data, toc_builder, builder_info) = serialize_test_data(format);
        unsafe {
            // Deserialize and verify
            let new_table =
                TableOfContents::deserialize_v2xx(data.as_ptr(), builder_info.table_size).unwrap();

            // Verify data matches original
            assert_eq!(new_table.entries.len(), toc_builder.entries.len());
            assert_eq!(new_table.blocks.len(), toc_builder.blocks.len());

            let has_hash = format != ToCFormat::FEF64NoHash
                && format != ToCFormat::Preset1NoHash
                && format != ToCFormat::Preset3NoHash;

            let has_block_offset =
                format != ToCFormat::Preset3NoHash && format != ToCFormat::Preset3;

            // Check each file entry
            for (x, original) in toc_builder.entries.iter().enumerate() {
                let deserialized = &new_table.entries[x];
                assert_file_entries_equal(original, deserialized, x, has_hash, has_block_offset);
            }

            // Check each block
            for (original, deserialized) in toc_builder.blocks.iter().zip(new_table.blocks.iter()) {
                assert_eq!(original.compressed_size, deserialized.compressed_size);
            }

            // Check block compressions
            for (original, deserialized) in toc_builder
                .block_compressions
                .iter()
                .zip(new_table.block_compressions.iter())
            {
                assert_eq!(original, deserialized);
            }
        }
    }

    #[cfg(feature = "hardened")]
    #[rstest]
    fn insufficient_data_for_header_returns_error() {
        // Try to deserialize with buffer that's too small for header
        let data = [0u8; 4]; // Less than 8 bytes needed for header

        unsafe {
            let result = TableOfContents::deserialize_v2xx(data.as_ptr(), 4);
            assert!(matches!(result,
                Err(DeserializeError::InsufficientData(e)) if e.available == 4 && e.expected == 8
            ));
        }
    }

    #[cfg(feature = "hardened")]
    #[rstest]
    fn insufficient_data_for_extended_header_returns_error() {
        // Create FEF64 header that requires extended header
        let mut data = [0u8; 12]; // Less than 16 bytes needed for extended header

        // Set FEF bit and bits requiring extended header
        // Force extended header by ensuring we need >42 bits.
        let header = Fef64TocHeader::new(
            true, // has_hash
            63,   // string_pool_size_bits
            63,   // file_count_bits
            63,   // block_count_bits
            63,   // offset_bits
            0,    // padding
        );

        unsafe {
            // Write header to buffer
            (data.as_mut_ptr() as *mut u64).write_unaligned(header.0);

            let result = TableOfContents::deserialize_v2xx(data.as_ptr(), 12);
            assert!(matches!(result,
                Err(DeserializeError::InsufficientData(e)) if e.available == 12 && e.expected == 16
            ));
        }
    }

    #[cfg(feature = "hardened")]
    #[rstest]
    #[case::preset0(ToCFormat::Preset0)]
    #[case::fef64(ToCFormat::FEF64)]
    fn insufficient_data_for_preset_toc_returns_error(#[case] format: ToCFormat) {
        let (data, _toc_builder, builder_info) = serialize_test_data(format);
        let toc_size = builder_info.table_size;

        // Try deserialize with buffer that's a byte too small for full ToC
        let truncated_size = toc_size - 1;

        unsafe {
            let result = TableOfContents::deserialize_v2xx(data.as_ptr(), truncated_size);
            assert!(matches!(result,
                Err(DeserializeError::InsufficientData(e)) if e.available == truncated_size && e.expected == toc_size
            ));
        }
    }
}
