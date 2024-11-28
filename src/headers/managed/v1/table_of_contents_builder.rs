use super::file_entry_intrinsics::{write_entries_as_v0, write_entries_as_v1};
use crate::{
    api::{enums::compression_preference::CompressionPreference, traits::*},
    headers::{enums::v1::*, managed::*, parser::*, raw::toc::*},
    implementation::pack::{
        blocks::polyfills::Block, table_of_contents_builder_state::TableOfContentsBuilderState,
    },
};
use endian_writer::{EndianWriter, LittleEndianWriter};
use std::alloc::Allocator;

// Max values for V0 & V1 formats.
const MAX_BLOCK_COUNT_V0V1: usize = 262143; // 2^18 - 1
const MAX_FILE_COUNT_V0V1: usize = 1048575; // 2^20 - 1

/// Determines the required Table of Contents version based on the largest file size in the given blocks.
///
/// This function iterates through all blocks to find the largest file size and checks if any block
/// can create chunks. It then determines the version based on whether the largest file size
/// exceeds [`u32::MAX`].
///
/// # Arguments
///
/// * `blocks` - A slice of [Box<dyn Block<T>>] representing the blocks in the archive.
///
/// # Returns
///
/// A [VersionInfo] struct containing the determined [TableOfContentsVersion] and a boolean
/// indicating if any block can create chunks.
///
/// # Type Parameters
///
/// * `T`: Type of the items in the blocks, which must implement `HasFileSize`, `CanProvideInputData`, and `HasRelativePath`.
pub fn determine_version<T>(blocks: &[Box<dyn Block<T>>], chunk_size: u32) -> VersionInfo
where
    T: HasFileSize + CanProvideInputData + HasRelativePath,
{
    let mut largest_file_size: u64 = 0;
    let mut can_create_chunks = false;

    for block in blocks {
        // Note: Some items will be counted multiple times due to chunked blocks.
        for item in block.items() {
            let file_size = item.file_size();
            if file_size > largest_file_size {
                largest_file_size = file_size;
            }

            if file_size > chunk_size as u64 {
                can_create_chunks = true;
            }
        }
    }

    let version = if largest_file_size > u32::MAX as u64 {
        TableOfContentsVersion::V1
    } else {
        TableOfContentsVersion::V0
    };

    VersionInfo {
        version,
        can_create_chunks,
    }
}

/// Packs a `StringPool` for a given `TableOfContentsVersion` and slice of files.
///
/// This function creates a `StringPool` containing the relative paths of all files,
/// using the appropriate format based on the specified `TableOfContentsVersion`.
///
/// # Arguments
///
/// * `version` - The `TableOfContentsVersion` to use for packing.
/// * `files` - A slice of items implementing `HasRelativePath`.
/// * `short_alloc` - The allocator for short term allocations (those that die with method scope).
/// * `long_alloc` - The allocator for long term allocations. (values that are returned)
///
/// # Returns
///
/// Returns a `Result` which is:
/// * `Ok(Vec<u8>)` containing the packed `StringPool` data on success.
/// * `Err(StringPoolPackError)` if an error occurs during packing.
///
/// # Remarks
///
/// How you get the `files` field depends on the nature of the request made to [pack_string_pool_with_allocators].
/// If we're packing original content only, then we have this from the list of files we're deciding to pack.
///
/// If we're 'repacking', i.e. packing with externally imported blocks not in the file list, we must then instead
/// concatenate the relative paths from these blocks onto the existing ones in new files.
///
/// These can be distinguished in higher level APIs 'pack_with_existing_blocks` vs `pack`.
pub fn pack_string_pool_with_allocators<
    T: HasRelativePath,
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
>(
    _version: TableOfContentsVersion,
    files: &mut [T],
    short_alloc: ShortAlloc,
    long_alloc: LongAlloc,
) -> Result<Vec<u8, LongAlloc>, StringPoolPackError> {
    StringPool::pack_v0_with_allocators(files, short_alloc, long_alloc, true)
}

/// Serializes the table of contents data into a binary format from a builder state.
///
/// This function takes the builder state of a table of contents and serializes it into a binary
/// format at the specified memory location. It handles the serialization of the header,
/// file entries, block information, and string pool data.
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
/// * `version` - The [TableOfContentsVersion] to use for serialization. Get this from calling [determine_version].
/// * `data_ptr` - A raw pointer to the memory where the data will be written.
/// * `raw_string_pool_data` - A slice containing the raw string pool data to be written. This should be obtained by calling [pack_string_pool_with_allocators] with the same version.
///
/// # Returns
///
/// Returns a `Result` which is:
/// - `Ok(usize)` containing the number of bytes written if serialization is successful.
/// - `Err(SerializeError)` if an error occurs during serialization.
///
/// # Remarks
///
/// Before calling this, first call [determine_version], construct a StringPool based on version
/// using [pack_string_pool_with_allocators], and pack the files such that [TableOfContentsBuilderState] contains valid data.
/// Then call this to finalize the pre-allocated space.
///
/// # Errors
///
/// This function will return an error if:
/// - The number of blocks exceeds [MAX_BLOCK_COUNT_V0V1].
/// - The number of file entries exceeds [MAX_FILE_COUNT_V0V1].
#[allow(dead_code)] // TODO: This is temporary
pub(crate) unsafe fn serialize_table_of_contents_from_state(
    builder_state: &TableOfContentsBuilderState,
    version: TableOfContentsVersion,
    data_ptr: *mut u8,
    raw_string_pool_data: &[u8],
) -> Result<usize, SerializeError> {
    serialize_table_of_contents(
        &builder_state.block_compressions,
        &builder_state.blocks,
        &builder_state.entries,
        version,
        data_ptr,
        raw_string_pool_data,
    )
}

/// Serializes the table of contents data into a binary format.
///
/// This function takes the components of a table of contents and serializes them into a binary
/// format at the specified memory location. It handles the serialization of the header,
/// file entries, block information, and string pool data.
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
/// * `version` - The [TableOfContentsVersion] to use for serialization. Get this from calling [determine_version].
/// * `data_ptr` - A raw pointer to the memory where the data will be written.
/// * `raw_string_pool_data` - A slice containing the raw string pool data to be written. This should be obtained by calling [pack_string_pool_with_allocators] with the same version.
///
/// # Returns
///
/// Returns a `Result` which is:
/// - `Ok(usize)` containing the number of bytes written if serialization is successful.
/// - `Err(SerializeError)` if an error occurs during serialization.
///
/// # Errors
///
/// This function will return an error if:
/// - The number of blocks exceeds [MAX_BLOCK_COUNT_V0V1].
/// - The number of file entries exceeds [MAX_FILE_COUNT_V0V1].
pub unsafe fn serialize_table_of_contents(
    block_compressions: &[CompressionPreference],
    blocks: &[BlockSize],
    entries: &[FileEntry],
    version: TableOfContentsVersion,
    data_ptr: *mut u8,
    raw_string_pool_data: &[u8],
) -> Result<usize, SerializeError> {
    if blocks.len() > MAX_BLOCK_COUNT_V0V1 {
        return Err(SerializeError::TooManyBlocks(blocks.len()));
    }

    if entries.len() > MAX_FILE_COUNT_V0V1 {
        return Err(SerializeError::TooManyFiles(entries.len()));
    }

    let mut writer = LittleEndianWriter::new(data_ptr);
    let header = NativeTocHeader::new(
        entries.len() as u32,
        blocks.len() as u32,
        raw_string_pool_data.len() as u32,
        version,
    );
    writer.write_u64(header.0);

    // Write the entries into the ToC header.
    if !entries.is_empty() {
        match version {
            TableOfContentsVersion::V0 => {
                write_entries_as_v0(&mut writer, entries);
            }
            TableOfContentsVersion::V1 => {
                write_entries_as_v1(&mut writer, entries);
            }
        }
    }

    // Now write the blocks after the headers.
    if !blocks.is_empty() {
        write_blocks(blocks, block_compressions, &mut writer);
    }

    // Write the raw string pool data.
    writer.write_bytes(raw_string_pool_data);

    Ok(writer.ptr as usize - data_ptr as usize)
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
            NativeV1TocBlockEntry::to_writer(num_blocks, compression, lewriter);
        }
    }

    // No benefit found from unrolling here.
}

/// Calculates the size of the table after serialization to binary.
/// This is used for pre-allocating space needed for the table.
///
/// # Arguments
///
/// * `num_entries` - Number of file entries in the table.
/// * `num_blocks` - Number of blocks in the table.
/// * `pool_len` - Length of the string pool.
/// * `version` - Version to serialize into.
///
/// # Returns
///
/// Size of the Table of Contents
pub fn calculate_table_size(
    num_entries: usize,
    num_blocks: usize,
    pool_len: usize,
    version: TableOfContentsVersion,
) -> usize {
    const HEADER_SIZE: usize = 8;
    let mut current_size = HEADER_SIZE;

    let entry_size = match version {
        TableOfContentsVersion::V0 => 20,
        TableOfContentsVersion::V1 => 24,
    };

    current_size += num_entries * entry_size;
    current_size += num_blocks * size_of::<NativeV1TocBlockEntry>();
    current_size += pool_len;

    current_size
}

/// Holds the result of version determination for a Table of Contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionInfo {
    /// The determined version of the Table of Contents.
    pub version: TableOfContentsVersion,
    /// Indicates whether any block can create chunks.
    pub can_create_chunks: bool,
}

/// Errors that can occur when serializing TableOfContents
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SerializeError {
    /// Too many blocks in the table of contents
    TooManyBlocks(usize),
    /// Too many files in the table of contents
    TooManyFiles(usize),
    /// Unsupported table of contents version
    UnsupportedVersion(TableOfContentsVersion),
}

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
