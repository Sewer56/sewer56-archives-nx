/*
use crate::api::traits::can_provide_file_data::CanProvideFileData;
use crate::api::traits::has_file_size::HasFileSize;
use crate::api::traits::has_relative_path::HasRelativePath;
use crate::headers::enums::table_of_contents_version::TableOfContentsVersion;
use crate::headers::managed::block_size::BlockSize;
use crate::headers::managed::file_entry::FileEntry;
use crate::headers::parser::string_pool::StringPool;
use crate::headers::parser::string_pool_common::StringPoolUnpackError;
use crate::headers::raw::native_toc_block_entry::NativeTocBlockEntry;
use crate::headers::raw::native_toc_header::NativeTocHeader;
use crate::utilities::serialize::*;
use crate::{
    api::enums::compression_preference::CompressionPreference,
    implementation::pack::blocks::polyfills::Block,
};
use alloc::rc::Rc;
use little_endian_writer::LittleEndianWriter;
use std::alloc::{Allocator, Global};

// Max values for V0 & V1 formats.
const MAX_BLOCK_COUNT_V0V1: usize = 262143; // 2^18 - 1
const MAX_FILE_COUNT_V0V1: usize = 1048575; // 2^20 - 1

/// This contains the shared 'state' used to build the final binary Table of Contents.
///
/// # Remarks
///
/// This item is passed (boxed) onto each block, which in turn is responsible for compressing its own
/// contents.
pub struct TableOfContentsBuilderState<LongAlloc: Allocator + Clone = Global> {
    /// Used formats for compression of each block.
    pub block_compressions: Box<[CompressionPreference], LongAlloc>,

    /// Individual block sizes in this structure.
    pub blocks: Box<[BlockSize], LongAlloc>,

    /// Individual file entries.
    pub entries: Box<[FileEntry], LongAlloc>,
}

impl<'a, TFile, ShortAlloc, LongAlloc> TableOfContentsBuilder<TFile, ShortAlloc, LongAlloc>
where
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
    TFile: HasRelativePath + HasFileSize + CanProvideFileData,
{
    /// Creates a new [TableOfContentsBuilder] for a given set of blocks and files with which
    /// the table of contents will be created from.
    ///
    /// # Arguments
    ///
    /// The following arguments are used to determine which `settings` to initialize the builder with:
    ///
    /// - blocks: The resulting blocks from the call to [`make_blocks`]
    ///           The blocks should contain the files, without duplicated files in any block.
    ///
    /// [`make_blocks`]: crate::utilities::arrange::pack::make_blocks::make_blocks
    pub fn new(blocks: &Vec<Box<dyn Block<TFile>>>) -> Self {}

    /// Serializes the ToC to allow reading from binary.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it works with raw pointers.
    ///
    /// # Arguments
    ///
    /// * `data_ptr` - Memory where to serialize to.
    /// * `version` - Version of the archive used.
    /// * `string_pool_data` - Raw data for the string pool.
    ///
    /// # Returns
    ///
    /// Returns a `Result` which is:
    /// - `Ok(usize)` containing the number of bytes written if serialization is successful.
    /// - `Err(SerializeError)` if an error occurs during serialization.
    ///
    /// # Remarks
    ///
    /// To determine needed size of `data_ptr`, call `calculate_table_size`.
    pub unsafe fn serialize(
        &self,
        data_ptr: *mut u8,
        version: TableOfContentsVersion,
        raw_string_pool_data: &[u8],
    ) -> Result<usize, SerializeError> {
        serialize_table_of_contents(
            &self.block_compressions,
            &self.blocks,
            &self.entries,
            version,
            data_ptr,
            raw_string_pool_data,
        )
    }

    /// Calculates the size of the table after serialization to binary.
    ///
    /// # Arguments
    ///
    /// * `version` - Version to serialize into.
    ///
    /// # Returns
    ///
    /// Size of the Table of Contents
    pub fn calculate_table_size(&self, version: TableOfContentsVersion) -> usize {
        calculate_table_size(
            self.entries.len(),
            self.blocks.len(),
            self.pool.len(),
            version,
        )
    }
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
/// * `version` - The `TableOfContentsVersion` to use for serialization.
/// * `data_ptr` - A raw pointer to the memory where the data will be written.
/// * `raw_string_pool_data` - A slice containing the raw string pool data to be written.
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
/// - The number of blocks exceeds `MAX_BLOCK_COUNT_V0V1`.
/// - The number of file entries exceeds `MAX_FILE_COUNT_V0V1`.
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
    let header = NativeTocHeader::init(
        entries.len() as u32,
        blocks.len() as u32,
        raw_string_pool_data.len() as u32,
        version,
    );
    writer.write(header.0);

    // Write the entries into the ToC header.
    if !entries.is_empty() {
        match version {
            TableOfContentsVersion::V0 => {
                for entry in entries {
                    entry.write_as_v0(&mut writer);
                }
            }
            TableOfContentsVersion::V1 => {
                for entry in entries {
                    entry.write_as_v1(&mut writer);
                }
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
    writer: &mut LittleEndianWriter,
) {
    // This makes the bounds checker leave us alone.
    debug_assert!(blocks.len() == compressions.len());

    // SAFETY: Debug&Test Builds Verify that both arrays have the same length
    //         They should have the same length by definition
    unsafe {
        for x in 0..blocks.len() {
            let num_blocks = (*blocks.as_ptr().add(x)).compressed_size;
            let compression = *compressions.as_ptr().add(x);
            let entry = NativeTocBlockEntry::new(num_blocks, compression);
            writer.write(entry.0);
        }
    }

    // Note: Unlike C#, unrolling is not needed. LLVM is clever enough to do it for us.
}

/// Calculates the size of the table after serialization to binary.
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
fn calculate_table_size(
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
    current_size += num_blocks * size_of::<NativeTocBlockEntry>();
    current_size += pool_len;

    current_size
}

/// Errors that can occur when deserializing TableOfContents
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeserializeError {
    /// Error unpacking the string pool
    StringPoolUnpackError(StringPoolUnpackError),
    /// Unsupported table of contents version
    UnsupportedTocVersion,
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
*/
