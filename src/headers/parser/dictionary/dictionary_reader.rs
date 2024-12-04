use crate::{
    headers::{managed::InsufficientDataError, parser::*, types::xxh3sum::XXH3sum},
    implementation::pack::blocks::polyfills::NO_DICTIONARY_INDEX,
    utilities::compression::*,
};
use crate::prelude::*;
use core::{
    alloc::Layout, ops::{Deref, DerefMut}, ptr::copy_nonoverlapping, slice
};
use derive_new::new;
use endian_writer::{ByteAlign, EndianReader, LittleEndianReader};
use safe_allocator_api::RawAlloc;
use thiserror_no_std::Error;

#[derive(Debug, Error)]
pub enum DictionaryReadError {
    #[error("Dictionary data is too large. Maximum decompressed size is 268,435,455 bytes.")]
    DecompressedSizeTooLarge,
    #[error("Failed to decompress dictionary data")]
    DecompressionError(#[from] NxDecompressionError),
    #[error("Failed to allocate memory")]
    AllocationError(#[from] AllocError),
    #[error("Invalid dictionary header")]
    InvalidHeader,
    #[error("Dictionary hash mismatch")]
    HashMismatch,
    #[error(
        "Insufficient data to read dictionary segment. Available: {}, Expected: {}",
        ._0.available,
        ._0.expected
    )]
    InsufficientData(#[from] InsufficientDataError),
    #[error("Decompressed size of dictionary is incorrect. Expected: {0}, Actual: {1}. This indicates a faulty file.")]
    DecompressedSizeMismatch(u32, u32),
    #[error("Decompressed payload must be at least as big as payload header (8 bytes).")]
    InsufficientPayloadSize,
    #[error("Dictionary index in payload header is out of bounds. Index: {0}. Num Dictionaries (Max Index): {1}. This indicates a faulty file.")]
    InvalidDictionaryIndex(u32, u32),
    #[error(
        "Invalid dictionary data. Block dictionary lengths go out of bound of total block count."
    )]
    InvalidDictionaryBlockLengthData,
}

#[derive(Clone, Copy)]
struct DictionaryRange {
    offset: u32,
    length: u32,
}

pub struct DictionaryData {
    /// Raw decompressed dictionary data
    raw_data: Box<[u8]>,

    /// Dictionary offset+length pairs
    dict_ranges: Box<[DictionaryRange]>,

    /// Block to dictionary mapping information
    dict_indices_for_block: Box<[u8]>,
}

impl DictionaryData {
    /// Gets dictionary data for a block at the specified index.
    /// This returns an empty slice if no dictionary is used.
    ///
    /// # Safety
    ///
    /// Caller must ensure block_index is valid if not using hardened mode
    pub unsafe fn get_dictionary_for_block_unchecked(&self, block_index: usize) -> &[u8] {
        // If the block index is out of range, return an empty slice
        #[cfg(feature = "hardened")]
        if block_index >= self.dict_indices_for_block.len() {
            return &[];
        }

        // SAFETY: This is safe provided that the DictionaryData itself is valid.
        // The hardening of deserialization logic should make this safe.
        let dict_index = *self.dict_indices_for_block.get_unchecked(block_index) as usize;
        if dict_index == NO_DICTIONARY_INDEX as usize {
            return &[];
        }

        let range = self.dict_ranges.get_unchecked(dict_index);
        self.raw_data
            .get_unchecked(range.offset as usize..(range.offset + range.length) as usize)
    }
}

/// Reads the main dictionary header [`DictionariesHeader`] and extracts the inner compressed dictionary
/// payload.
///
/// # Arguments
/// * `dictionary_data` - Slice of compressed dictionary data.
///                       This begins at the dictionary header [`DictionariesHeader`] and must be
///                       at least as long as the length of the dictionary segment.
///
/// # Remarks
///
/// The argument passed as `dictionary_data` may be longer than the dictionary segment itself,
/// its length is not relevant, as long as it's long enough.
///
/// # Safety
///
/// This function is safe if ran in 'hardened' mode, else it is unsafe to call on untrusted data.
pub unsafe fn extract_payload_with_allocator<ShortAlloc: Allocator + Clone>(
    dictionary_data: &[u8],
    short_alloc: ShortAlloc,
) -> Result<ExtractPayloadResult<ShortAlloc>, DictionaryReadError> {
    // Validate we have enough bytes for the header
    #[cfg(feature = "hardened")]
    if dictionary_data.len() < DictionariesHeader::SIZE_BYTES {
        return Err(InsufficientDataError::new(dictionary_data.len() as u32, DictionariesHeader::SIZE_BYTES as u32).into());
    }

    let mut reader = LittleEndianReader::new(dictionary_data.as_ptr());
    let header = DictionariesHeader(reader.read_u64());

    // Validate we have enough bytes for the content
    let decompressed_size = header.decompressed_size();
    let compressed_size = header.compressed_size();

    #[cfg(feature = "hardened")]
    {
        // For uncompressed data (compressed_size == 0), we need decompressed_size bytes
        // For compressed data, we need compressed_size bytes
        let required_size = if compressed_size == 0 {
            decompressed_size
        } else {
            compressed_size
        };

        // Ensure we have enough bytes for the content
        let remaining_bytes = dictionary_data.len() - DictionariesHeader::SIZE_BYTES;
        if remaining_bytes < required_size as usize {
            return Err(InsufficientDataError::new(
                dictionary_data.len() as u32,
                required_size + DictionariesHeader::SIZE_BYTES as u32,
            )
            .into());
        }

        // Ensure decompressed data is large enough
        if decompressed_size < DictionariesPayloadHeader::SIZE_BYTES as u32 {
            return Err(DictionaryReadError::InsufficientPayloadSize);
        }
    }

    // Allocate space for the decompressed/copied data
    let layout = Layout::from_size_align_unchecked(decompressed_size as usize, 8);
    #[allow(unused_mut)]
    let mut decompressed_data: Aligned8RawAlloc<ShortAlloc> =
        RawAlloc::new_in(layout, short_alloc.clone())
            .unwrap()
            .into();

    if compressed_size == 0 {
        // Data is not compressed - copy it directly
        #[cfg(feature = "hardened")]
        {
            if dictionary_data.len() < DictionariesHeader::SIZE_BYTES + decompressed_size as usize {
                return Err(InsufficientDataError::new(
                    dictionary_data.len() as u32,
                    (DictionariesHeader::SIZE_BYTES + decompressed_size as usize) as u32,
                ).into());
            }
        }
        
        unsafe {
            copy_nonoverlapping(
                reader.ptr,
                decompressed_data.as_mut_ptr(),
                decompressed_size as usize
            );
        }
    } else {
        // Data is compressed - decompress it
        let _decompressed_bytes = zstd::decompress(
            slice::from_raw_parts(reader.ptr, compressed_size as usize),
            decompressed_data.as_mut_slice(),
        )?;

        #[cfg(feature = "hardened")]
        if _decompressed_bytes != decompressed_size as usize {
            return Err(DictionaryReadError::DecompressedSizeMismatch(
                decompressed_size,
                _decompressed_bytes as u32,
            ));
        }
    }

    Ok(ExtractPayloadResult::new(decompressed_data))
}

/// Contains the result of extracting the payload from the pickled header
#[derive(Debug, new)]
pub struct ExtractPayloadResult<ShortAlloc: Allocator + Clone> {
    pub(crate) decompressed_data: Aligned8RawAlloc<ShortAlloc>,
}

/// Extracts the inner compressed payload from the call to [`extract_payload_with_allocator`].
///
/// # Arguments
/// * `payload_result` - The result of the call to [`extract_payload_with_allocator`].
///
/// # Returns
///
/// A ready to use dictionary data object.
///
/// # Safety
///
/// This function is safe if ran in 'hardened' mode, else it is unsafe to call on untrusted data.
pub unsafe fn parse_payload_with_allocator<ShortAlloc: Allocator + Clone>(
    payload_result: &ExtractPayloadResult<ShortAlloc>,
) -> Result<DictionaryData, DictionaryReadError> {
    let mut reader = LittleEndianReader::new(payload_result.decompressed_data.as_ptr());
    let header = DictionariesPayloadHeader(reader.read_u64());

    // Validate the payload data is sufficiently big
    #[cfg(feature = "hardened")]
    {
        let decompressed_data_len = payload_result.decompressed_data.len();
        let expected_size = calculate_payload_header_size(
            header.num_mappings(),
            header.num_dictionaries(),
            header.has_hashes() != 0,
        );
        if decompressed_data_len < expected_size as usize {
            return Err(
                InsufficientDataError::new(decompressed_data_len as u32, expected_size).into(),
            );
        }
    }

    // SAFETY: Max value of this field is `u22::MAX`, so by definition this can use max
    // 4MB of RAM
    let mut dict_indices_for_block: Box<[u8]> =
        Box::new_uninit_slice(header.last_dict_block_index() as usize).assume_init();

    // Prepare to ingest the dictionary data
    let mut dict_index_reader = LittleEndianReader::new(reader.ptr);
    let mut dict_length_reader = LittleEndianReader::new(reader.ptr);
    dict_length_reader.seek(header.num_mappings() as isize);

    // Fill in the mapping
    let mut dict_indices_ptr = dict_indices_for_block.as_mut_ptr();
    for _ in 0..header.num_mappings() {
        let dict_index = dict_index_reader.read_u8();
        let num_items = dict_length_reader.read_u8();

        // Validate the dictionary index is in range.
        #[cfg(feature = "hardened")]
        if dict_index as usize >= header.num_dictionaries() as usize
            && dict_index != NO_DICTIONARY_INDEX
        {
            return Err(DictionaryReadError::InvalidDictionaryIndex(
                dict_index as u32,
                header.num_dictionaries(),
            ));
        }

        // Validate that the dict_indices_ptr won't overflow on writing num_items
        // Note: This is written with `usize` because `miri` complains about UB when using as_ptr
        // despite no actual read being done.
        #[cfg(feature = "hardened")]
        {
            let write_end_ptr = dict_indices_ptr as usize + num_items as usize;
            let allocated_end_ptr = dict_indices_for_block.as_ptr() as usize
                + dict_indices_for_block.len();
            if write_end_ptr > allocated_end_ptr {
                return Err(DictionaryReadError::InvalidDictionaryBlockLengthData);
            }
        }

        for _ in 0..num_items {
            *dict_indices_ptr = dict_index;
            dict_indices_ptr = dict_indices_ptr.add(1);
        }
    }

    // Validate that all dict_indices have been written to
    #[cfg(feature = "hardened")]
    if dict_indices_ptr as *const u8
        != dict_indices_for_block
            .as_ptr()
            .add(dict_indices_for_block.len())
    {
        return Err(DictionaryReadError::InvalidDictionaryBlockLengthData);
    }

    // Align 32 bits
    // SAFETY: The decompressed payload is aligned to 8 bytes, therefore this is a valid way to align.
    dict_length_reader.align_power_of_two(4);
    let mut dict_sizes_reader = dict_length_reader;
    let mut dict_ranges: Box<[DictionaryRange]> =
        Box::new_uninit_slice(header.num_dictionaries() as usize).assume_init();
    let mut current_offset = 0;

    for x in 0..dict_ranges.len() {
        let dict_size = dict_sizes_reader.read_u32();
        dict_ranges[x] = DictionaryRange {
            offset: current_offset,
            length: dict_size,
        };
        current_offset += dict_size;
    }

    // Skip hashes if present
    if header.has_hashes() != 0 {
        // First align to 8
        dict_sizes_reader.align_power_of_two(8);
        let hashes_size = size_of::<XXH3sum>() * header.num_dictionaries() as usize;
        dict_sizes_reader.ptr = (dict_sizes_reader.ptr as usize + hashes_size) as *const u8;
    }

    // Assert there are enough bytes for the payload
    let raw_data_start = dict_sizes_reader.ptr as usize;
    let used_bytes = raw_data_start - payload_result.decompressed_data.as_ptr() as usize;
    let remaining_bytes = payload_result.decompressed_data.len() - used_bytes;

    #[cfg(feature = "hardened")]
    {
        let expected_data_size = current_offset as usize;
        if remaining_bytes < expected_data_size {
            return Err(
                InsufficientDataError::new(remaining_bytes as u32, expected_data_size as u32).into(),
            );
        }
    }


    let raw_data = slice::from_raw_parts(raw_data_start as *const u8, remaining_bytes);

    Ok(DictionaryData {
        raw_data: Box::from(raw_data),
        dict_ranges,
        dict_indices_for_block,
    })
}

#[derive(Debug)]
pub struct Aligned8RawAlloc<ShortAlloc: Allocator>(RawAlloc<ShortAlloc>);

impl<ShortAlloc: Allocator> Deref for Aligned8RawAlloc<ShortAlloc> {
    type Target = RawAlloc<ShortAlloc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<ShortAlloc: Allocator> DerefMut for Aligned8RawAlloc<ShortAlloc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<ShortAlloc: Allocator> From<RawAlloc<ShortAlloc>> for Aligned8RawAlloc<ShortAlloc> {
    fn from(raw_alloc: RawAlloc<ShortAlloc>) -> Self {
        Self(raw_alloc)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;
    use crate::{
        implementation::pack::blocks::polyfills::NO_DICTIONARY_INDEX,
        utilities::tests::mock_block::create_mock_block,
    };
    use allocator_api2::vec;

    #[rstest]
    #[case(false)]
    #[cfg_attr(not(miri), case(true))]
    fn can_serialize_deserialize_basic_dictionary(#[case] compress: bool) {
        // Create test data
        let dict1: Vec<u8> = (0..100).collect(); // Dictionary with values 0-99
        let dict2: Vec<u8> = (0..50).collect(); // Dictionary with values 0-49
        let dictionaries = vec![dict1.as_slice(), dict2.as_slice()];

        // Create blocks that use these dictionaries
        let blocks = vec![
            create_mock_block(0),
            create_mock_block(0),
            create_mock_block(0),
            create_mock_block(1),
            create_mock_block(1),
            create_mock_block(0),
            create_mock_block(0),
        ];

        // Serialize the dictionary data
        let serialized = serialize_dictionary_data(&dictionaries, &blocks, true, compress).unwrap();

        // Deserialize and verify the data
        let deserialized = unsafe { deserialize_dictionary_data(&serialized).unwrap() };

        // Verify dictionary content for each block
        unsafe {
            // First three blocks should use dictionary 0
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(0), &dict1);
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(1), &dict1);
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(2), &dict1);

            // Next two blocks should use dictionary 1
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(3), &dict2);
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(4), &dict2);

            // Last two blocks should use dictionary 0
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(5), &dict1);
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(6), &dict1);
        }
    }

    #[rstest]
    #[case(false)]
    #[cfg_attr(not(miri), case(true))]
    fn can_serialize_deserialize_empty_dictionaries(#[case] compress: bool) {
        // Create test data with empty dictionaries
        let dict1: Vec<u8> = vec![];
        let dict2: Vec<u8> = vec![];
        let dictionaries = vec![dict1.as_slice(), dict2.as_slice()];

        // Create blocks that use these dictionaries
        let blocks = vec![
            create_mock_block(0),
            create_mock_block(1),
            create_mock_block(0),
        ];

        // Serialize the dictionary data
        let serialized = serialize_dictionary_data(&dictionaries, &blocks, true, compress).unwrap();

        // Deserialize and verify the data
        let deserialized = unsafe { deserialize_dictionary_data(&serialized).unwrap() };

        // Verify dictionary content for each block
        unsafe {
            assert!(deserialized.get_dictionary_for_block_unchecked(0).is_empty());
            assert!(deserialized.get_dictionary_for_block_unchecked(1).is_empty());
            assert!(deserialized.get_dictionary_for_block_unchecked(2).is_empty());
        }
    }

    #[rstest]
    #[case(false)]
    #[cfg_attr(not(miri), case(true))]
    fn can_serialize_deserialize_with_no_dictionary_blocks(#[case] compress: bool) {
        // Create test data
        let dict1: Vec<u8> = (0..100).collect();
        let dictionaries = vec![dict1.as_slice()];

        // Create blocks with some having no dictionary
        let blocks = vec![
            create_mock_block(0),
            create_mock_block(NO_DICTIONARY_INDEX as u32),
            create_mock_block(0),
            create_mock_block(NO_DICTIONARY_INDEX as u32),
            create_mock_block(0),
        ];

        // Serialize the dictionary data
        let serialized = serialize_dictionary_data(&dictionaries, &blocks, true, compress).unwrap();

        // Deserialize and verify the data
        let deserialized = unsafe { deserialize_dictionary_data(&serialized).unwrap() };

        // Verify dictionary content for each block
        unsafe {
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(0), &dict1);
            assert!(deserialized.get_dictionary_for_block_unchecked(1).is_empty());
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(2), &dict1);
            assert!(deserialized.get_dictionary_for_block_unchecked(3).is_empty());
            assert_eq!(deserialized.get_dictionary_for_block_unchecked(4), &dict1);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "hardened")]
mod invalid_data_tests {
    use rstest::rstest;
    use crate::utilities::tests::mock_block::create_mock_block;
    use super::*;
    use allocator_api2::vec;    

    #[test]
    fn insufficient_dict_header_bytes() {
        // Create data that's too short for even the header
        let data = vec![1, 2, 3, 4, 5, 6, 7]; // Less than DictionariesHeader::SIZE_BYTES

        let result = unsafe { deserialize_dictionary_data(&data) };
        assert!(matches!(result, 
            Err(DictionaryReadError::InsufficientData(err)) if err.available == 7 && err.expected == DictionariesHeader::SIZE_BYTES as u32));
    }

    #[rstest]
    #[case(false)]
    #[cfg_attr(not(miri), case(true))]
    fn insufficient_compressed_data(#[case] compress: bool) {
        // Create valid dictionary data but truncate it
        let dict1: Vec<u8> = vec![1, 2, 3];
        let dictionaries = vec![dict1.as_slice()];
        let blocks = vec![create_mock_block(0)];
        
        let mut serialized = serialize_dictionary_data(&dictionaries, &blocks, true, compress).unwrap();
        serialized.truncate(serialized.len() - 1); // Truncate to create insufficient data scenario
        
        let result = unsafe { deserialize_dictionary_data(&serialized) };
        assert!(matches!(result, Err(DictionaryReadError::InsufficientData(_))));
    }

    #[rstest]
    #[case(false)]
    #[cfg_attr(not(miri), case(true))]
    fn insufficient_payload_header_size(#[case] compress: bool) {
        // Create very small dictionary data that would decompress to less than header size
        let dict1: Vec<u8> = vec![];
        let dictionaries = vec![dict1.as_slice()];
        let blocks = vec![create_mock_block(0)];
        
        let mut serialized = serialize_dictionary_data(&dictionaries, &blocks, true, compress).unwrap();
        
        // Corrupt the decompressed size in the header to be too small
        let mut header = DictionariesHeader(u64::from_le_bytes(serialized[0..8].try_into().unwrap()));
        header.set_decompressed_size(DictionariesPayloadHeader::SIZE_BYTES as u32 - 1); // Too small for payload header
        serialized[0..8].copy_from_slice(&header.0.to_le_bytes());
        
        let result = unsafe { deserialize_dictionary_data(&serialized) };
        assert!(matches!(result, Err(DictionaryReadError::InsufficientPayloadSize)));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn decompressed_size_too_big() {
        let dict1: Vec<u8> = vec![1, 2, 3];
        let dictionaries = vec![dict1.as_slice()];
        let blocks = vec![create_mock_block(0)];
        
        let mut serialized = serialize_dictionary_data(&dictionaries, &blocks, true, true).unwrap();
        
        // Corrupt the decompressed size in the header
        let mut header = DictionariesHeader(u64::from_le_bytes(serialized[0..8].try_into().unwrap()));
        header.set_decompressed_size(1000); // Much larger than actual
        serialized[0..8].copy_from_slice(&header.0.to_le_bytes());
        
        let result = unsafe { deserialize_dictionary_data(&serialized) };
        assert!(matches!(result, 
            Err(DictionaryReadError::DecompressedSizeMismatch(expected, _actual)) if expected == 1000));
    }

    #[rstest]
    #[case(false)]
    #[cfg_attr(not(miri), case(true))]
    fn invalid_dictionary_index(#[case] compress: bool) {
        // Create dictionary data with invalid dictionary index
        let dict1: Vec<u8> = vec![1, 2, 3];
        let dictionaries = vec![dict1.as_slice()];
        
        // Create blocks referencing invalid dictionary index
        let blocks = vec![
            create_mock_block(2), // Invalid index, only 1 dictionary exists
        ];
        
        let serialized = serialize_dictionary_data(&dictionaries, &blocks, true, compress).unwrap();
        let result = unsafe { deserialize_dictionary_data(&serialized) };
        assert!(matches!(result, 
            Err(DictionaryReadError::InvalidDictionaryIndex(index, num_dicts)) if index == 2 && num_dicts == 1));
    }

    #[rstest]
    #[case(false)]
    #[cfg_attr(not(miri), case(true))]
    fn out_of_bounds_block_access(#[case] compress: bool) {
        let dict1: Vec<u8> = vec![1, 2, 3];
        let dictionaries = vec![dict1.as_slice()];
        let blocks = vec![create_mock_block(0)];
        
        let serialized = serialize_dictionary_data(&dictionaries, &blocks, true, compress).unwrap();
        let dictionary_data = unsafe { deserialize_dictionary_data(&serialized) }.unwrap();
        
        // Try to access block beyond the end
        let result = unsafe { dictionary_data.get_dictionary_for_block_unchecked(100) };
        assert!(result.is_empty(), "Out of bounds access should return empty slice");
    }

    #[test]
    fn invalid_block_length_data() {
        let dict1: Vec<u8> = vec![1, 2, 3];
        let dictionaries = vec![dict1.as_slice()];
        let blocks = vec![create_mock_block(0)];
        
        let mut serialized = serialize_dictionary_data(&dictionaries, &blocks, true, false).unwrap();
        
        // Extract the payload to modify it
        let mut payload_result = unsafe { 
            extract_payload_with_allocator(&serialized, Global).unwrap()
        };

        // Modify the block length to be invalid (too large)
        let decompressed_slice = payload_result.decompressed_data.as_mut_slice();
        decompressed_slice[9] = 255; // Modify first BlockDictionaryLength entry (see specification)

        // Reconstruct serialized data (no compression)
        let header = DictionariesHeader::new(
            0, 
            0, 
            0,
            decompressed_slice.len() as u32
        );
        serialized = Vec::new();
        serialized.extend_from_slice(&header.0.to_le_bytes());
        serialized.extend_from_slice(decompressed_slice);
        
        let result = unsafe { deserialize_dictionary_data(&serialized) };
        assert!(matches!(result, Err(DictionaryReadError::InvalidDictionaryBlockLengthData)));
    }

    #[test]
    fn insufficient_payload_size() {
        let dict1: Vec<u8> = vec![1, 2, 3];
        let dictionaries = vec![dict1.as_slice()];
        let blocks = vec![create_mock_block(0)];
        
        let mut serialized = serialize_dictionary_data(&dictionaries, &blocks, true, false).unwrap();
        
        // Extract the payload to modify it
        let mut payload_result = unsafe { 
            extract_payload_with_allocator(&serialized, Global).unwrap()
        };

        // Modify payload header to claim more dictionaries than available
        let decompressed_slice = payload_result.decompressed_data.as_mut_slice();
        let mut payload_header = DictionariesPayloadHeader(
            u64::from_le_bytes(decompressed_slice[0..8].try_into().unwrap())
        );
        payload_header.set_num_dictionaries(1000); // Much larger than available data
        payload_header.set_num_mappings(2000);
        decompressed_slice[0..8].copy_from_slice(&payload_header.0.to_le_bytes());

        // Create new header with correct sizes
        let header = DictionariesHeader::new(
            0, 
            0, 
            0,
            decompressed_slice.len() as u32
        );
        serialized = Vec::new();
        serialized.extend_from_slice(&header.0.to_le_bytes());
        serialized.extend_from_slice(decompressed_slice);
        
        let result = unsafe { deserialize_dictionary_data(&serialized) };
        assert!(matches!(result, Err(DictionaryReadError::InsufficientData(_))));
    }

    #[test]
    fn insufficient_raw_data_content() {
        let dict1: Vec<u8> = vec![1, 2, 3];
        let dictionaries = vec![dict1.as_slice()];
        let blocks = vec![create_mock_block(0)];
        
        let mut serialized = serialize_dictionary_data(&dictionaries, &blocks, true, false).unwrap();
        
        // Extract the payload to modify it
        let mut payload_result = unsafe { 
            extract_payload_with_allocator(&serialized, Global).unwrap()
        };

        // Get the decompressed data
        let decompressed_slice = payload_result.decompressed_data.as_mut_slice();
        
        // Modify the dictionary size to be larger than available data [set DictionarySizes[0] to be very large, see spec)
        // Dictionary sizes start after block mappings, aligned to 4 bytes
        let dict_size_offset = 12; // 8 bytes header + 2 bytes (1 index, 1 legth) + aligned to 4 = 12
        let dict_size_bytes = &mut decompressed_slice[dict_size_offset..dict_size_offset + 4];
        dict_size_bytes.copy_from_slice(&(1000000u32.to_le_bytes())); // Set dictionary size to be very large

        // Create new header with correct sizes
        let header = DictionariesHeader::new(
            0, 
            0, 
            0,
            decompressed_slice.len() as u32
        );
        serialized = Vec::new();
        serialized.extend_from_slice(&header.0.to_le_bytes());
        serialized.extend_from_slice(decompressed_slice);
        
        let result = unsafe { deserialize_dictionary_data(&serialized) };
        assert!(matches!(result, Err(DictionaryReadError::InsufficientData(_))));
    }
}
