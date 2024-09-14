use crate::api::enums::compression_preference::CompressionPreference;
use crate::headers::enums::table_of_contents_version::TableOfContentsVersion;
use crate::headers::managed::block_size::BlockSize;
use crate::headers::managed::file_entry::FileEntry;
use crate::headers::parser::string_pool::StringPool;
use crate::headers::parser::string_pool_common::StringPoolUnpackError;
use crate::headers::raw::native_toc_block_entry::NativeTocBlockEntry;
use crate::headers::raw::native_toc_header::NativeTocHeader;
use crate::utilities::serialize::*;
use little_endian_reader::LittleEndianReader;
use little_endian_writer::LittleEndianWriter;
use std::alloc::{Allocator, Global};
use std::mem;
use std::slice;

/// Managed representation of the deserialized table of contents.
pub struct TableOfContents<
    ShortAlloc: Allocator + Clone = Global,
    LongAlloc: Allocator + Clone = Global,
> {
    /// Used formats for compression of each block.
    pub block_compressions: Vec<CompressionPreference>,

    /// Individual block sizes in this structure.
    pub blocks: Vec<BlockSize>,

    /// Individual file entries.
    pub entries: Vec<FileEntry>,

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
    // Max values for V0 & V1 formats.
    const MAX_BLOCK_COUNT_V0V1: usize = 262143; // 2^18 - 1
    const MAX_FILE_COUNT_V0V1: usize = 1048575; // 2^20 - 1

    /*
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
    pub unsafe fn deserialize(data_ptr: *const u8) -> Result<Self, DeserializeError>
    where
        T: Default,
    {
        let mut reader = LittleEndianReader::new(data_ptr);
        let toc_header = NativeTocHeader::from_raw(reader.read::<u64>());

        if toc_header.block_count() as usize > Self::MAX_BLOCK_COUNT_V0V1 {
            return Err(DeserializeError::TooManyBlocks(
                toc_header.block_count() as usize
            ));
        }

        if toc_header.file_count() as usize > Self::MAX_FILE_COUNT_V0V1 {
            return Err(DeserializeError::TooManyFiles(
                toc_header.file_count() as usize
            ));
        }

        let mut entries = vec![FileEntry::default(); toc_header.file_count() as usize];
        let mut blocks = vec![BlockSize::default(); toc_header.block_count() as usize];
        let mut block_compressions =
            vec![CompressionPreference::NoPreference; toc_header.block_count() as usize];

        if !entries.is_empty() {
            match toc_header.get_version() {
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

        Self::read_blocks_unrolled(&mut blocks, &mut block_compressions, &mut reader);

        let pool = StringPool::unpack(
            slice::from_raw_parts(reader.ptr(), toc_header.string_pool_size() as usize),
            toc_header.file_count() as usize,
            StringPool::get_format_from_version(toc_header.get_version()),
        )
        .map_err(DeserializeError::StringPoolUnpackError)?;

        Ok(TableOfContents {
            block_compressions,
            blocks,
            entries,
            pool,
            version: toc_header.get_version(),
        })
    }

    /// Helper function to read blocks in an unrolled manner for performance.
    fn read_blocks_unrolled(
        blocks: &mut [BlockSize],
        compressions: &mut [CompressionPreference],
        reader: &mut LittleEndianReader,
    ) {
        let mut blocks_iter = blocks.iter_mut();
        let mut compressions_iter = compressions.iter_mut();

        while blocks_iter.len() >= 4 {
            for _ in 0..4 {
                let value = reader.read::<u32>();
                if let (Some(block), Some(compression)) =
                    (blocks_iter.next(), compressions_iter.next())
                {
                    block.compressed_size = value >> 3;
                    *compression = unsafe { mem::transmute((value & 0x7) as u8) };
                }
            }
        }

        for (block, compression) in blocks_iter.zip(compressions_iter) {
            let value = reader.read::<u32>();
            block.compressed_size = value >> 3;
            *compression = unsafe { mem::transmute((value & 0x7) as u8) };
        }
    }
    */

    /// Serializes the ToC to allow reading from binary.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it works with raw pointers.
    ///
    /// # Arguments
    ///
    /// * `data_ptr` - Memory where to serialize to.
    /// * `toc_size` - Size of table of contents.
    /// * `version` - Version of the archive used.
    /// * `string_pool_data` - Raw data for the string pool.
    ///
    /// # Returns
    ///
    /// Result containing the number of bytes written or a SerializeError.
    ///
    /// # Remarks
    ///
    /// To determine needed size of `data_ptr`, call `calculate_table_size`.
    pub unsafe fn serialize(
        &self,
        data_ptr: *mut u8,
        toc_size: usize,
        version: TableOfContentsVersion,
        raw_string_pool_data: &[u8],
    ) -> Result<usize, SerializeError> {
        if self.blocks.len() > Self::MAX_BLOCK_COUNT_V0V1 {
            return Err(SerializeError::TooManyBlocks(self.blocks.len()));
        }

        if self.entries.len() > Self::MAX_FILE_COUNT_V0V1 {
            return Err(SerializeError::TooManyFiles(self.entries.len()));
        }

        let mut writer = LittleEndianWriter::new(data_ptr);
        let header = NativeTocHeader::init(
            self.entries.len() as u32,
            self.blocks.len() as u32,
            raw_string_pool_data.len() as u32,
            version,
        );
        writer.write(header.0);

        // Write the entries into the ToC header.
        if !self.entries.is_empty() {
            match version {
                TableOfContentsVersion::V0 => {
                    for entry in &self.entries {
                        entry.write_as_v0(&mut writer);
                    }
                }
                TableOfContentsVersion::V1 => {
                    for entry in &self.entries {
                        entry.write_as_v1(&mut writer);
                    }
                }
            }
        }

        // Now write the blocks after the headers.
        if !self.blocks.is_empty() {
            write_blocks(&self.blocks, &self.block_compressions, &mut writer);
        }

        // Write the raw string pool data.
        writer.write_bytes(raw_string_pool_data);

        Ok(writer.ptr as usize - data_ptr as usize)
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
        const HEADER_SIZE: usize = 8;
        let mut current_size = HEADER_SIZE;

        let entry_size = match version {
            TableOfContentsVersion::V0 => 20,
            TableOfContentsVersion::V1 => 24,
        };

        current_size += self.entries.len() * entry_size;
        current_size += self.blocks.len() * 4;
        current_size += self.pool.len();

        current_size
    }
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

/// Errors that can occur when deserializing TableOfContents
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeserializeError {
    /// Too many blocks in the table of contents
    TooManyBlocks(usize),
    /// Too many files in the table of contents
    TooManyFiles(usize),
    /// Error unpacking the string pool
    StringPoolUnpackError(StringPoolUnpackError),
    /// Unsupported table of contents version
    UnsupportedVersion(TableOfContentsVersion),
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

/*
impl<T> PartialEq for TableOfContents<T> {
    fn eq(&self, other: &Self) -> bool {
        self.block_compressions == other.block_compressions
            && self.blocks == other.blocks
            && self.entries == other.entries
            && self.pool == other.pool
            && self.version == other.version
    }
}

impl<T> Eq for TableOfContents<T> {}

impl<T> std::hash::Hash for TableOfContents<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pool.len().hash(state);
    }
}
*/
