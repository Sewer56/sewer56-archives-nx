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
        ToCFormat::Preset0 => Ok(serialize_table_of_contents_preset0(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            0,
        )),
        ToCFormat::Preset1NoHash => Ok(serialize_table_of_contents_preset0(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            1,
        )),
        ToCFormat::Preset2 => Ok(serialize_table_of_contents_preset0(
            block_compressions,
            blocks,
            entries,
            info,
            data_ptr,
            2,
        )),
        ToCFormat::Preset3 => todo!(),
        ToCFormat::Preset3NoHash => todo!(),
        ToCFormat::Error => unreachable_unchecked(),
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
    let item_counts = ItemCounts::new(string_pool_size_bits, file_count_bits, block_count_bits);
    if include_hash {
        for item in entries {
            let entry16 = FileEntry16::from_file_entry(item_counts, item);
            entry16.to_writer(&mut lewriter);
        }
    } else {
        for item in entries {
            let entry8 = FileEntry8::from_file_entry(item_counts, item);
            entry8.to_writer(&mut lewriter);
        }
    }

    // Now write the blocks after the headers.
    if !blocks.is_empty() {
        write_blocks(blocks, block_compressions, &mut lewriter);
    }

    // Write the raw string pool data.
    lewriter.write_bytes(&info.string_pool);
    lewriter.ptr as usize - data_ptr as usize
}

unsafe fn serialize_table_of_contents_preset0<LongAlloc: Allocator + Clone>(
    block_compressions: &[CompressionPreference],
    blocks: &[BlockSize],
    entries: &[FileEntry],
    info: &BuilderInfo<LongAlloc>,
    data_ptr: *mut u8,
    preset_no: u8,
) -> usize {
    let mut lewriter = LittleEndianWriter::new(data_ptr);

    // Write the initial header.
    let header = Preset0TocHeader::new(
        preset_no,
        info.string_pool.len() as u32,
        blocks.len() as u32,
        entries.len() as u32,
    );
    lewriter.write_u64(header.0);

    // Serialize the entries.
    if preset_no == 0 {
        lewriter.write_entries_into_unroll_2::<NativeFileEntryP0, FileEntry>(entries);
    } else if preset_no == 1 {
    } else {
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
pub enum SerializeError {}

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
fn calculate_toc_size(
    format: ToCFormat,
    string_pool_len: u32,
    block_count: u32,
    file_count: u32,
) -> u32 {
    let mut toc_size = 0;
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

    toc_size += entry_size * entry_size;
    toc_size += block_count * size_of::<NativeV2TocBlockEntry>() as u32;
    toc_size += string_pool_len;
    toc_size
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::packing::packing_settings::MAX_BLOCK_SIZE;
    use crate::utilities::tests::packer_file_for_testing::PackerFileForTesting;
    use rstest::rstest;

    #[rstest]
    #[case(TableOfContentsVersion::V0)]
    #[case(TableOfContentsVersion::V1)]
    fn can_serialize_and_deserialize(#[case] version: TableOfContentsVersion) {
        // Note: We're not testing the actual file/chunk creation logic, this data can be whatever.
        let files = [
            PackerFileForTesting::new_rc("dvdroot/textures/s01.txd", 113763968),
            PackerFileForTesting::new_rc("dvdroot/textures/s12.txd", 62939496),
            PackerFileForTesting::new_rc("ModConfig.json", 768),
            PackerFileForTesting::new_rc("Readme.md", 3072),
            PackerFileForTesting::new_rc("Changelog.md", 2048),
        ];

        // Generate dummy data for archived file.
        let entries: Vec<FileEntry> = vec![
            FileEntry::new(0xBBBBBBBBBBBBBBBB, 113763968, 0, 0, 0),
            FileEntry::new(0xCCCCCCCCCCCCCCCC, 62939496, 0, 1, 2),
            FileEntry::new(0xDDDDDDDDDDDDDDDD, 768, 0, 2, 4),
            FileEntry::new(0xEEEEEEEEEEEEEEEE, 3072, 768, 3, 4),
            FileEntry::new(0xFFFFFFFFFFFFFFFF, 2048, 3840, 4, 4),
        ];

        // Generate blocks with pre-calculated values.
        let blocks: Vec<BlockSize> = vec![
            BlockSize::new(113763968),
            BlockSize::new(62939496),
            BlockSize::new(768),
            BlockSize::new(3072 + 2048),
        ];

        let block_compressions: Vec<CompressionPreference> = vec![
            CompressionPreference::Copy,
            CompressionPreference::ZStandard,
            CompressionPreference::Lz4,
            CompressionPreference::Copy,
        ];

        // Generate TOC.
        let mut toc_builder =
            unsafe { TableOfContentsBuilderState::new(blocks.len(), files.len()) };

        // Emulate a write to entries by the packing operation.
        for (x, entry) in entries.iter().enumerate() {
            toc_builder.set_entry(x, *entry).unwrap();
        }

        for (x, block) in blocks.iter().enumerate() {
            toc_builder.set_block(x, *block).unwrap();
        }

        for (x, compression) in block_compressions.iter().enumerate() {
            toc_builder.set_block_compression(x, *compression).unwrap();
        }

        // Serialize
        let data_size = calculate_table_size(
            files.len(),
            blocks.len(),
            0, // Assuming string pool size is 0 for simplicity
            version,
        );
        let mut data = vec![0u8; data_size];

        unsafe {
            let bytes_written = serialize_table_of_contents_from_state(
                &toc_builder,
                version,
                data.as_mut_ptr(),
                &[], // Empty string pool for simplicity
            )
            .unwrap();
            assert_eq!(bytes_written, data_size);

            // Deserialize
            let new_table = TableOfContents::deserialize_v1xx(data.as_ptr()).unwrap();

            // Compare deserialized data with original
            assert_eq!(new_table.entries.len(), entries.len());
            assert_eq!(new_table.blocks.len(), blocks.len());

            // Check each file entry
            for (x, original) in entries.iter().enumerate() {
                let deserialized = &new_table.entries[x];
                assert_file_entries_equal(original, deserialized, x);
            }

            // Check each block
            for (original, deserialized) in blocks.iter().zip(new_table.blocks.iter()) {
                assert_eq!(original.compressed_size, deserialized.compressed_size);
            }

            // Check block compressions
            for (original, deserialized) in block_compressions
                .iter()
                .zip(new_table.block_compressions.iter())
            {
                assert_eq!(original, deserialized);
            }
        }
    }

    #[rstest]
    #[case(TableOfContentsVersion::V0)]
    #[case(TableOfContentsVersion::V1)]
    #[cfg_attr(all(miri, not(feature = "miri_extra_checks")), ignore)]
    fn can_serialize_maximum_file_count_v0_v1(#[case] version: TableOfContentsVersion) {
        // Generate maximum number of dummy file entries
        let entries: Vec<FileEntry> = generate_file_entries(MAX_FILE_COUNT_V0V1);

        // Generate TOC
        let mut toc_builder = unsafe { TableOfContentsBuilderState::new(1, entries.len()) };

        // Populate the TOC builder
        for (x, entry) in entries.iter().enumerate() {
            toc_builder.set_entry(x, *entry).unwrap();
        }

        // Add a single dummy block
        toc_builder
            .set_block(0, BlockSize::new(MAX_BLOCK_SIZE))
            .unwrap();
        toc_builder
            .set_block_compression(0, CompressionPreference::Copy)
            .unwrap();

        // Serialize
        let data_size = calculate_table_size(
            entries.len(),
            1, // Single block
            0, // Assuming string pool size is 0 for simplicity
            version,
        );
        let mut data = vec![0u8; data_size];

        unsafe {
            let result = serialize_table_of_contents_from_state(
                &toc_builder,
                version,
                data.as_mut_ptr(),
                &[], // Empty string pool for simplicity
            );

            assert!(
                result.is_ok(),
                "Serialization failed for maximum file count"
            );
            let bytes_written = result.unwrap();
            assert_eq!(
                bytes_written, data_size,
                "Incorrect number of bytes written"
            );

            // Deserialize to verify
            let new_table = TableOfContents::deserialize_v1xx(data.as_ptr()).unwrap();

            // Verify deserialized data
            assert_eq!(new_table.entries.len(), MAX_FILE_COUNT_V0V1);

            // Check each file entry
            for (x, original) in entries.iter().enumerate() {
                let deserialized = &new_table.entries[x];
                assert_file_entries_equal(original, deserialized, x);
            }
        }
    }

    #[rstest]
    #[case(TableOfContentsVersion::V0)]
    #[case(TableOfContentsVersion::V1)]
    #[cfg_attr(all(miri, not(feature = "miri_extra_checks")), ignore)]
    fn throws_error_when_file_count_exceeds_maximum(#[case] version: TableOfContentsVersion) {
        // Generate one more than the maximum number of dummy file entries
        let entries: Vec<FileEntry> = generate_file_entries(MAX_FILE_COUNT_V0V1 + 1);

        // Generate TOC
        let mut toc_builder = unsafe { TableOfContentsBuilderState::new(1, entries.len()) };

        // Populate the TOC builder
        for (x, entry) in entries.iter().enumerate() {
            toc_builder.set_entry(x, *entry).unwrap();
        }

        // Add a single dummy block
        toc_builder
            .set_block(0, BlockSize::new(MAX_BLOCK_SIZE))
            .unwrap();
        toc_builder
            .set_block_compression(0, CompressionPreference::Copy)
            .unwrap();

        // Attempt to serialize
        let data_size = calculate_table_size(entries.len(), 1, 0, version);
        let mut data = vec![0u8; data_size];

        unsafe {
            let result = serialize_table_of_contents_from_state(
                &toc_builder,
                version,
                data.as_mut_ptr(),
                &[], // Empty string pool for simplicity
            );

            // Assert that the result is an error
            assert!(
                result.is_err(),
                "Expected SerializeError, but serialization succeeded"
            );

            // Check that it's the correct error type
            match result {
                Err(SerializeError::TooManyFiles(count)) => {
                    assert_eq!(
                        count,
                        MAX_FILE_COUNT_V0V1 + 1,
                        "Incorrect file count in error"
                    );
                }
                _ => panic!(
                    "Expected SerializeError::TooManyFiles, but got a different error or success"
                ),
            }
        }
    }

    #[rstest]
    #[case(TableOfContentsVersion::V0)]
    #[case(TableOfContentsVersion::V1)]
    #[cfg_attr(all(miri, not(feature = "miri_extra_checks")), ignore)]
    fn can_serialize_maximum_block_count(#[case] version: TableOfContentsVersion) {
        let blocks = generate_blocks(MAX_BLOCK_COUNT_V0V1);
        let block_compressions = generate_block_compressions(MAX_BLOCK_COUNT_V0V1);

        // Generate a single dummy file entry
        let entries = [FileEntry::new(0, 1024, 0, 0, 0)];

        // Generate TOC
        let mut toc_builder =
            unsafe { TableOfContentsBuilderState::new(blocks.len(), entries.len()) };

        // Populate the TOC builder
        toc_builder.set_entry(0, entries[0]).unwrap();

        for (x, block) in blocks.iter().enumerate() {
            toc_builder.set_block(x, *block).unwrap();
        }

        for (x, compression) in block_compressions.iter().enumerate() {
            toc_builder.set_block_compression(x, *compression).unwrap();
        }

        // Serialize
        let data_size = calculate_table_size(
            entries.len(),
            blocks.len(),
            0, // Assuming string pool size is 0 for simplicity
            version,
        );
        let mut data = vec![0u8; data_size];

        unsafe {
            let result = serialize_table_of_contents_from_state(
                &toc_builder,
                version,
                data.as_mut_ptr(),
                &[], // Empty string pool for simplicity
            );

            assert!(
                result.is_ok(),
                "Serialization failed for maximum block count"
            );
            let bytes_written = result.unwrap();
            assert_eq!(
                bytes_written, data_size,
                "Incorrect number of bytes written"
            );

            // Deserialize to verify
            let new_table = TableOfContents::deserialize_v1xx(data.as_ptr()).unwrap();

            // Verify deserialized data
            assert_eq!(new_table.blocks.len(), MAX_BLOCK_COUNT_V0V1);
            assert_eq!(new_table.block_compressions.len(), MAX_BLOCK_COUNT_V0V1);

            // Verify all blocks
            for (x, original) in blocks.iter().enumerate() {
                let deserialized = &new_table.blocks[x];
                assert_eq!(
                    original.compressed_size, deserialized.compressed_size,
                    "Mismatch in compressed_size for block {}",
                    x
                );
                assert!(
                    deserialized.compressed_size <= MAX_BLOCK_SIZE,
                    "Block size exceeds MAX_BLOCK_SIZE for block {}",
                    x
                );
            }

            // Verify all block compressions
            for (x, original) in block_compressions.iter().enumerate() {
                let deserialized = &new_table.block_compressions[x];
                assert_eq!(
                    original, deserialized,
                    "Mismatch in compression preference for block {}",
                    x
                );
            }
        }
    }

    #[rstest]
    #[case(TableOfContentsVersion::V0)]
    #[case(TableOfContentsVersion::V1)]
    #[cfg_attr(all(miri, not(feature = "miri_extra_checks")), ignore)]
    fn throws_error_when_block_count_exceeds_maximum(#[case] version: TableOfContentsVersion) {
        let blocks = generate_blocks(MAX_BLOCK_COUNT_V0V1 + 1);
        let block_compressions = generate_block_compressions(MAX_BLOCK_COUNT_V0V1 + 1);

        // Generate a single dummy file entry
        let entries = [FileEntry::new(0, 1024, 0, 0, 0)];

        // Generate TOC
        let mut toc_builder =
            unsafe { TableOfContentsBuilderState::new(blocks.len(), entries.len()) };

        // Populate the TOC builder
        toc_builder.set_entry(0, entries[0]).unwrap();

        for (x, block) in blocks.iter().enumerate() {
            toc_builder.set_block(x, *block).unwrap();
        }

        for (x, compression) in block_compressions.iter().enumerate() {
            toc_builder.set_block_compression(x, *compression).unwrap();
        }

        // Attempt to serialize
        let data_size = calculate_table_size(
            entries.len(),
            blocks.len(),
            0, // Assuming string pool size is 0 for simplicity
            version,
        );
        let mut data = vec![0u8; data_size];

        unsafe {
            let result = serialize_table_of_contents_from_state(
                &toc_builder,
                version,
                data.as_mut_ptr(),
                &[], // Empty string pool for simplicity
            );

            // Assert that the result is an error
            assert!(
                result.is_err(),
                "Expected SerializeError, but serialization succeeded"
            );

            // Check that it's the correct error type
            match result {
                Err(SerializeError::TooManyBlocks(count)) => {
                    assert_eq!(
                        count,
                        MAX_BLOCK_COUNT_V0V1 + 1,
                        "Incorrect block count in error"
                    );
                }
                _ => panic!(
                    "Expected SerializeError::TooManyBlocks, but got a different error or success"
                ),
            }
        }
    }

    fn generate_file_entries(count: usize) -> Vec<FileEntry> {
        (0..count)
            .map(|x| {
                FileEntry::new(
                    // Using index as hash for simplicity
                    x as u64,
                    // All files are up to as big as block for simplicity
                    ((x * 1024) % MAX_BLOCK_SIZE as usize) as u64,
                    // Clamped offset calculation
                    ((x * 1024) % MAX_BLOCK_SIZE as usize) as u32,
                    x as u32,
                    // All files in the first block
                    0,
                )
            })
            .collect()
    }

    fn assert_file_entries_equal(original: &FileEntry, deserialized: &FileEntry, index: usize) {
        assert_eq!(
            original.hash, deserialized.hash,
            "Mismatch in hash for entry {}",
            index
        );
        assert_eq!(
            original.decompressed_size, deserialized.decompressed_size,
            "Mismatch in decompressed_size for entry {}",
            index
        );
        assert_eq!(
            original.decompressed_block_offset, deserialized.decompressed_block_offset,
            "Mismatch in decompressed_block_offset for entry {}",
            index
        );
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

    fn generate_blocks(count: usize) -> Vec<BlockSize> {
        (0..count)
            .map(|x| {
                let size = ((x * 1024) % MAX_BLOCK_SIZE as usize) as u64; // Vary block size, 1KB to 1MB
                BlockSize::new((size.min(MAX_BLOCK_SIZE as u64)) as u32)
            })
            .collect()
    }

    // The generate_block_compressions function remains unchanged
    fn generate_block_compressions(count: usize) -> Vec<CompressionPreference> {
        (0..count)
            .map(|x| {
                if x % 2 == 0 {
                    CompressionPreference::ZStandard
                } else {
                    CompressionPreference::Lz4
                }
            })
            .collect()
    }
}
 */
