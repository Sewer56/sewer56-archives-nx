use crate::{
    api::traits::*,
    headers::{raw::toc::PRESET0_BLOCK_COUNT_MAX, types::xxh3sum::XXH3sum},
    implementation::pack::blocks::polyfills::NO_DICTIONARY_INDEX,
    utilities::compression,
};
use alloc::alloc::*;
use bitfield::bitfield;
use core::{alloc::Layout, slice};
use derive_new::new;
use endian_writer::{ByteAlign, EndianWriter, LittleEndianWriter};
use safe_allocator_api::RawAlloc;
use thiserror_no_std::Error;

/// Maximum number of possible dictionaries
pub const MAX_DICTIONARIES: usize = 254;

/// Using a more aggressive compression level since
/// we're dealing with a small amount of data.
const DEFAULT_COMPRESSION_LEVEL: i32 = 16;

/// Maximum amount of blocks that can be serialized in a dictionary.
pub const MAX_BLOCK_COUNT: u32 = PRESET0_BLOCK_COUNT_MAX;

/// Error type for dictionary serialization operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum DictionarySerializeError {
    #[error("Too many dictionaries specified. Maximum is 254.")]
    TooManyDictionaries,
    #[error("Too many mappings specified. Maximum is u24::MAX.")]
    TooManyMappings,
    #[error("Dictionary data is too large. Maximum compressed size is 134,217,727 bytes.")]
    CompressedSizeTooLarge,
    #[error("Dictionary data is too large. Maximum decompressed size is 268,435,455 bytes.")]
    DecompressedSizeTooLarge,
    #[error("No blocks provided.")]
    NoBlocks,
    #[error("Too many blocks provided. Maximum is u24::MAX.")]
    TooManyBlocks,
    #[error(transparent)]
    AllocationError(#[from] AllocError),
    #[error("Failed to compress dictionary data {0}")]
    CompressionError(#[from] compression::NxCompressionError),
}

/// Serializes dictionary data into a binary format.
///
/// # Arguments
/// * `dictionaries` - Raw dictionary data for each dictionary
/// * `blocks` - The blocks in the exact order they will be compressed in the archive.
/// * `short_alloc` - Allocator for temporary allocations. (lifetime of method)
/// * `write_hashes` - Whether to write the dictionary hashes
///
/// # Returns
/// Slice of compressed bytes.
pub fn serialize_dictionary_payload_with_allocator<THasDictIndex, ShortAlloc>(
    dictionaries: &[&[u8]],
    blocks: &[THasDictIndex],
    short_alloc: ShortAlloc,
    write_hashes: bool,
) -> Result<DictionarySerializeResult, DictionarySerializeError>
where
    THasDictIndex: HasDictIndex,
    ShortAlloc: Allocator + Clone,
{
    // Validate input
    if dictionaries.len() > MAX_DICTIONARIES {
        return Err(DictionarySerializeError::TooManyDictionaries);
    }

    if blocks.is_empty() {
        return Err(DictionarySerializeError::NoBlocks);
    }

    if blocks.len() > MAX_BLOCK_COUNT as usize {
        return Err(DictionarySerializeError::TooManyBlocks);
    }

    // Calculate compressed and decompressed sizes
    let mut last_block_index_with_dictionary = 0;
    let mappings = create_dictionary_mappings_with_allocator(
        blocks,
        short_alloc.clone(),
        &mut last_block_index_with_dictionary,
    )?;

    // SAFETY: It's technically impossible for mappings.len to overflow here, because worst case is 1 mapping
    //         per block and the block count is less than u22::MAX (4M). We error above in case that ever changes however.
    let header_size = calculate_payload_header_size(
        mappings.len() as u32,
        dictionaries.len() as u32,
        write_hashes,
    );
    let decompressed_size = header_size + dictionaries.iter().map(|d| d.len() as u32).sum::<u32>();

    // Allocate the buffer for the uncompressed data
    unsafe {
        // Allocate aligned to 8 bytes, to make future align operations on writer cheap.
        let layout = Layout::from_size_align_unchecked(decompressed_size as usize, 8);
        let mut decompressed_data = RawAlloc::new_in(layout, short_alloc.clone())?;

        let start_ptr = decompressed_data.as_mut_ptr();
        let mut writer = LittleEndianWriter::new(start_ptr);

        // Write item counts
        writer.write(
            &DictionariesPayloadHeader::new(
                dictionaries.len() as u32,
                mappings.len() as u32,
                last_block_index_with_dictionary,
                write_hashes,
            )
            .0,
        );

        // Write the mapping indices
        for mapping in &mappings {
            writer.write(&mapping.dictionary_index);
        }

        // Write the mapping lengths
        for mapping in &mappings {
            writer.write(&mapping.num_blocks);
        }

        // Align 32 bits
        // SAFETY: The allocation itself was initially aligned to 8 bytes, therefore aligns up to
        // 8 can be done directly on the memory address.
        writer.align_power_of_two(4);

        // Write the dictionary sizes
        for dictionary in dictionaries {
            writer.write_u32(dictionary.len() as u32);
        }

        if write_hashes {
            // Align 64 bits
            // SAFETY: The allocation itself was initially aligned to 8 bytes, therefore aligns up to
            // 8 can be done directly on the memory address.
            writer.align_power_of_two(8);

            // Write the dictionary hashes
            for dictionary in dictionaries {
                writer.write(&XXH3sum::create(dictionary));
            }
        }

        // Assert that we've written the expected number of bytes
        debug_assert_eq!(
            writer.ptr as usize - start_ptr as usize,
            header_size as usize
        );

        // Write the dictionary data
        for dictionary in dictionaries {
            writer.write_bytes(dictionary);
        }

        let num_written_bytes = writer.ptr as usize - start_ptr as usize;

        // Compress the data.
        let max_compressed_size = compression::zstd::max_alloc_for_compress_size(num_written_bytes);
        let mut compressed_data: Vec<u8> = Vec::with_capacity(max_compressed_size);
        let num_written_bytes = compression::zstd::compress_no_copy_fallback(
            DEFAULT_COMPRESSION_LEVEL,
            slice::from_raw_parts(start_ptr, num_written_bytes),
            slice::from_raw_parts_mut(compressed_data.as_mut_ptr(), compressed_data.capacity()),
        )?;

        // Slice the compressed data
        compressed_data.set_len(num_written_bytes);
        Ok(DictionarySerializeResult::new(
            DictionariesHeader::new(0, 0, compressed_data.len() as u32, decompressed_size),
            compressed_data,
        ))
    }
}

/// The result of a successful dictionary serialization.
#[derive(PartialEq, Eq, Debug, new)]
pub struct DictionarySerializeResult {
    /// The header to place before the payload.
    pub dict_header: DictionariesHeader,
    /// The payload to write after the header.
    pub payload: Vec<u8>,
}

/// Calculates the size of the dictionary payload in bytes (excluding the 8-byte header).
///
/// The size is calculated based on the format specification:
/// - 8 byte payload header
/// - num_mappings * (1 byte for BlockDictionaryIndex + 1 byte for BlockDictionaryLength)
/// - Padding to 32-bit alignment
/// - num_dictionaries * 4 bytes for DictionarySizes
///
/// if has_hashes:
///     - Padding to 64-bit alignment
///     - num_dictionaries * 8 bytes for DictionaryHashes
///
/// # Arguments
/// * `num_mappings` - Number of block mappings
/// * `num_dictionaries` - Number of dictionaries in the archive
/// * `has_hashes` - Whether the archive has hashes
///
/// # Returns
/// The total size in bytes needed for the dictionary payload (excluding header and raw dictionary data)
pub fn calculate_payload_header_size(
    num_mappings: u32,
    num_dictionaries: u32,
    has_hashes: bool,
) -> u32 {
    let mut size = 0u32;

    // DictionariesPayloadHeader
    size += DictionariesPayloadHeader::SIZE_BYTES as u32;

    // BlockDictionaryIndex (1 byte) + BlockDictionaryLength (1 byte) for each mapping
    size += num_mappings * 2;

    // Align to 32 bits (4 bytes)
    size = (size + 3) & !3;

    // DictionarySizes array (4 bytes per dictionary)
    size += num_dictionaries * 4;

    if has_hashes {
        // Align to 64 bits (8 bytes)
        size = (size + 7) & !7;

        // DictionaryHashes array (8 bytes per dictionary)
        size += num_dictionaries * 8;
    }

    size
}

/// Structure that represents a mapping between a dictionary index and a number of sequential blocks
/// for that given index.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DictionaryMapping {
    dictionary_index: u8,
    num_blocks: u8,
}

/// Creates a compressed list of dictionary mappings from a sequence of blocks.
///
/// This function processes blocks to generate dictionary index and count mappings by:
/// 1. Reading the dictionary index from each block
/// 2. Combining consecutive blocks that use the same dictionary
/// 3. Splitting runs longer than 255 blocks due to u8 size limitation
///
/// # Arguments
/// * `blocks` - Slice of blocks implementing the Block trait
/// * `short_alloc` - Allocator for temporary allocations. (lifetime of method)
/// * `last_block_with_dictionary` - Pointer to the last block with a dictionary
///
/// # Returns
/// * `Result` containing vector of mappings or DictionarySerializeError
///
/// # Errors
/// * `TooManyMappings` if the resulting number of mappings exceeds u24::MAX
pub(crate) fn create_dictionary_mappings_with_allocator<THasDictIndex, ShortAlloc>(
    blocks: &[THasDictIndex],
    short_alloc: ShortAlloc,
    last_block_with_dictionary_count: &mut u32,
) -> Result<Vec<DictionaryMapping, ShortAlloc>, DictionarySerializeError>
where
    THasDictIndex: HasDictIndex,
    ShortAlloc: Allocator + Clone,
{
    let mut mappings = Vec::with_capacity_in(8, short_alloc.clone());
    let mut current_index = u8::MAX; // Use reserved index as sentinel
    let mut current_count = 0u8;
    let mut num_total_blocks = 0_usize;

    for block in blocks {
        let dict_index = block.dict_index() as u8;

        // If they match, and we didn't hit max, then
        // just bump the count.
        if dict_index == current_index && current_count < u8::MAX {
            current_count += 1;
        } else {
            num_total_blocks += current_count as usize;
            if current_count > 0 {
                mappings.push(DictionaryMapping {
                    dictionary_index: current_index,
                    num_blocks: current_count,
                });

                if current_index != NO_DICTIONARY_INDEX {
                    *last_block_with_dictionary_count = num_total_blocks as u32;
                }
            }
            current_index = dict_index;
            current_count = 1;
        }
    }

    // Flush final mapping if any blocks were processed
    if current_count > 0 {
        num_total_blocks += current_count as usize;
        mappings.push(DictionaryMapping {
            dictionary_index: current_index,
            num_blocks: current_count,
        });

        if current_index != NO_DICTIONARY_INDEX {
            *last_block_with_dictionary_count = num_total_blocks as u32;
        }
    }

    // Verify we haven't exceeded maximum mappings
    if mappings.len() > 0x00FFFFFF {
        return Err(DictionarySerializeError::TooManyMappings);
    }

    Ok(mappings)
}

bitfield! {
    /// Packed header data
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct DictionariesHeader(u64);
    impl Debug;
    u32;

    /// `u5` Unused
    pub u8, reserved, set_reserved: 63, 59;
    /// `u4` The version of the archive
    pub u8, version, set_version: 59, 56;
    /// `u27` The compressed size of this archive
    pub compressed_size, set_compressed_size: 55, 28;
    /// `u28` The decompressed size of this archive
    pub decompressed_size, set_decompressed_size: 27, 0;
}

impl DictionariesHeader {
    pub const SIZE_BYTES: usize = size_of::<DictionariesHeader>();

    pub fn new(reserved: u8, version: u8, compressed_size: u32, decompressed_size: u32) -> Self {
        let mut header = DictionariesHeader(0);
        header.set_reserved(reserved);
        header.set_version(version);
        header.set_compressed_size(compressed_size);
        header.set_decompressed_size(decompressed_size);
        header
    }
}

bitfield! {
    /// Packed header data
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct DictionariesPayloadHeader(u64);
    impl Debug;
    u32;

    /// `u1` The flag for having a hash.
    pub u8, has_hashes, set_has_hashes: 52, 52;
    /// `u8` The number of dictionaries in this archive.
    pub num_dictionaries, set_num_dictionaries: 51, 44;
    /// `u22` The number of mappings in this archive.
    pub num_mappings, set_num_mappings: 43, 22;
    /// `u22` The last block index which uses a dictionary
    pub last_dict_block_index, set_last_dict_block_index: 21, 0;
}

impl DictionariesPayloadHeader {
    pub const SIZE_BYTES: usize = size_of::<DictionariesPayloadHeader>();

    pub fn new(
        num_dictionaries: u32,
        num_mappings: u32,
        last_dict_block_index: u32,
        has_hashes: bool,
    ) -> Self {
        let mut header = DictionariesPayloadHeader(0);
        header.set_num_dictionaries(num_dictionaries);
        header.set_num_mappings(num_mappings);
        header.set_last_dict_block_index(last_dict_block_index);
        header.set_has_hashes(has_hashes as u8);
        header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        implementation::pack::blocks::polyfills::Block,
        utilities::tests::{
            mock_block::create_mock_block, packer_file_for_testing::PackerFileForTesting,
        },
    };

    #[test]
    fn calc_size_can_calculate_min_size_dictionary() {
        // Test with minimum values (0 mappings, 0 dictionaries)
        let size = calculate_payload_header_size(0, 0, true);
        // 8 (num_dict + num_mappings + last_block_index)
        // Aligned to 8 bytes = 8
        assert_eq!(size, 8);
    }

    #[test]
    fn calc_size_can_handle_single_mapping_and_dictionary() {
        // Test with 1 mapping and 1 dictionary
        let size = calculate_payload_header_size(1, 1, true);
        // 4 (num_dict + num_mappings + last_block_index) + 2 (mapping) = 10
        // Aligned to 4 bytes = 12
        // + 4 (dictionary size) = 16
        // Aligned to 8 bytes = 16
        // + 8 (dictionary hash) = 24
        assert_eq!(size, 24);
    }

    #[test]
    fn calc_size_can_handle_multiple_mappings_and_dictionaries() {
        // Test with 5 mappings and 3 dictionaries
        let size = calculate_payload_header_size(5, 3, true);
        // 8 (num_dict + num_mappings + last_block_index) + 10 (5 mappings * 2) = 18
        // Aligned to 4 bytes = 20
        // + 12 (3 dictionaries * 4 bytes for sizes) = 32
        // Aligned to 8 bytes = 32
        // + 24 (3 dictionaries * 8 bytes for hashes) = 56
        assert_eq!(size, 56);
    }

    #[test]
    fn calc_size_handles_alignment_padding_correctly() {
        // Test with 3 mappings and 2 dictionaries to check alignment padding
        let size = calculate_payload_header_size(3, 2, true);
        // 8 (num_dict + num_mappings + last_block_index) + 6 (3 mappings * 2) = 14
        // Aligned to 4 bytes = 16
        // + 8 (2 dictionaries * 4 bytes for sizes) = 24
        // Aligned to 8 bytes = 24
        // + 16 (2 dictionaries * 8 bytes for hashes) = 40
        assert_eq!(size, 40);
    }

    #[test]
    fn calc_size_can_handle_large_dictionary_counts() {
        // Test with larger but valid values
        let size = calculate_payload_header_size(1000, 100, true);
        // 8 (num_dict + num_mappings + last_block_index) + 2000 (1000 mappings * 2) = 2008
        // Aligned to 4 bytes = 2008
        // + 400 (100 dictionaries * 4 bytes for sizes) = 2408
        // Aligned to 8 bytes = 2408
        // + 800 (100 dictionaries * 8 bytes for hashes) = 3208
        assert_eq!(size, 3208);
    }

    #[test]
    fn create_mappings_can_create_single_mapping() {
        let blocks = vec![
            create_mock_block(1),
            create_mock_block(1),
            create_mock_block(1),
        ];

        let mut last_block_idx = 0;
        let mappings = create_dictionary_mappings(&blocks, &mut last_block_idx).unwrap();
        assert_eq!(
            mappings,
            vec![DictionaryMapping {
                dictionary_index: 1,
                num_blocks: 3
            }]
        );
        assert_eq!(last_block_idx, 3); // All blocks use dictionary
    }

    #[test]
    fn create_mappings_can_handle_different_dictionaries() {
        let blocks = vec![
            create_mock_block(1),
            create_mock_block(1),
            create_mock_block(2),
            create_mock_block(2),
        ];

        let mut last_block_idx = 0;
        let mappings = create_dictionary_mappings(&blocks, &mut last_block_idx).unwrap();
        assert_eq!(
            mappings,
            vec![
                DictionaryMapping {
                    dictionary_index: 1,
                    num_blocks: 2
                },
                DictionaryMapping {
                    dictionary_index: 2,
                    num_blocks: 2
                },
            ]
        );
        assert_eq!(last_block_idx, 4); // All blocks use dictionary
    }

    #[test]
    fn create_mappings_handles_max_block_count_split() {
        let mut blocks = Vec::new();
        for _ in 0..300 {
            blocks.push(create_mock_block(1));
        }

        let mut last_block_idx = 0;
        let mappings = create_dictionary_mappings(&blocks, &mut last_block_idx).unwrap();
        assert_eq!(
            mappings,
            vec![
                DictionaryMapping {
                    dictionary_index: 1,
                    num_blocks: 255
                },
                DictionaryMapping {
                    dictionary_index: 1,
                    num_blocks: 45
                },
            ]
        );
        assert_eq!(last_block_idx, 300); // All blocks use dictionary
    }

    #[test]
    fn create_mappings_can_handle_empty_block_list() {
        let blocks: Vec<Box<dyn Block<PackerFileForTesting>>> = vec![];
        let mut last_block_idx = 0;
        let mappings = create_dictionary_mappings(&blocks, &mut last_block_idx).unwrap();
        assert_eq!(mappings.len(), 0);
        assert_eq!(last_block_idx, 0); // No blocks use dictionary
    }

    #[test]
    fn create_mappings_can_handle_mixed_dictionary_sequence() {
        let blocks = vec![
            create_mock_block(1),
            create_mock_block(1),
            create_mock_block(2),
            create_mock_block(1),
            create_mock_block(1),
        ];

        let mut last_block_idx = 0;
        let mappings = create_dictionary_mappings(&blocks, &mut last_block_idx).unwrap();
        assert_eq!(
            mappings,
            vec![
                DictionaryMapping {
                    dictionary_index: 1,
                    num_blocks: 2
                },
                DictionaryMapping {
                    dictionary_index: 2,
                    num_blocks: 1
                },
                DictionaryMapping {
                    dictionary_index: 1,
                    num_blocks: 2
                },
            ]
        );
        assert_eq!(last_block_idx, 5); // All blocks use dictionary
    }

    #[test]
    fn create_mappings_handles_no_dictionary_blocks() {
        let blocks = vec![
            create_mock_block(1),
            create_mock_block(NO_DICTIONARY_INDEX as u32),
            create_mock_block(2),
            create_mock_block(NO_DICTIONARY_INDEX as u32),
            create_mock_block(NO_DICTIONARY_INDEX as u32),
        ];

        let mut last_block_idx = 0;
        let mappings = create_dictionary_mappings(&blocks, &mut last_block_idx).unwrap();
        assert_eq!(
            mappings,
            vec![
                DictionaryMapping {
                    dictionary_index: 1,
                    num_blocks: 1
                },
                DictionaryMapping {
                    dictionary_index: NO_DICTIONARY_INDEX,
                    num_blocks: 1
                },
                DictionaryMapping {
                    dictionary_index: 2,
                    num_blocks: 1
                },
                DictionaryMapping {
                    dictionary_index: NO_DICTIONARY_INDEX,
                    num_blocks: 2
                },
            ]
        );
        assert_eq!(last_block_idx, 3); // Last block with dictionary is at count 3
    }

    #[test]
    fn create_mappings_handles_trailing_dictionary_blocks() {
        let blocks = vec![
            create_mock_block(NO_DICTIONARY_INDEX as u32),
            create_mock_block(NO_DICTIONARY_INDEX as u32),
            create_mock_block(1),
            create_mock_block(2),
            create_mock_block(2),
        ];

        let mut last_block_idx = 0;
        let mappings = create_dictionary_mappings(&blocks, &mut last_block_idx).unwrap();
        assert_eq!(
            mappings,
            vec![
                DictionaryMapping {
                    dictionary_index: NO_DICTIONARY_INDEX,
                    num_blocks: 2
                },
                DictionaryMapping {
                    dictionary_index: 1,
                    num_blocks: 1
                },
                DictionaryMapping {
                    dictionary_index: 2,
                    num_blocks: 2
                },
            ]
        );
        assert_eq!(last_block_idx, 5); // Last block with dictionary is at the end
    }

    pub(crate) fn create_dictionary_mappings<THasDictIndex>(
        blocks: &[THasDictIndex],
        last_block_with_dictionary_count: &mut u32,
    ) -> Result<Vec<DictionaryMapping>, DictionarySerializeError>
    where
        THasDictIndex: HasDictIndex,
    {
        create_dictionary_mappings_with_allocator(blocks, Global, last_block_with_dictionary_count)
    }
}
