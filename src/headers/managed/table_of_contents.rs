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
use little_endian_writer::LittleEndianWriter;
use std::alloc::{Allocator, Global};

// Max values for V0 & V1 formats.
const MAX_BLOCK_COUNT_V0V1: usize = 262143; // 2^18 - 1
const MAX_FILE_COUNT_V0V1: usize = 1048575; // 2^20 - 1

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

impl<ShortAlloc, LongAlloc> TableOfContents<ShortAlloc, LongAlloc>
where
    ShortAlloc: Allocator + Clone,
    LongAlloc: Allocator + Clone,
{
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
            Box::new_uninit_slice_in(toc_header.file_count() as usize, long_alloc.clone())
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

    current_size += (num_entries as usize) * entry_size;
    current_size += (num_blocks as usize) * size_of::<NativeTocBlockEntry>();
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
