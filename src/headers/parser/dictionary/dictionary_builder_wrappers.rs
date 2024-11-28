use super::{dictionary_builder::*, dictionary_reader::*};
use crate::api::traits::*;
use std::alloc::{Allocator, Global};

/// Serializes compressed dictionary payload.
///
/// # Arguments
/// * `dictionaries` - Raw dictionary data for each dictionary
/// * `blocks` - The blocks in the exact order they will be compressed in the archive.
/// * `write_hashes` - Whether to write the dictionary hashes
///
/// # Returns
///
/// A result containing the compressed payload, and the header that should precede it.
pub fn serialize_dictionary_payload<THasDictIndex>(
    dictionaries: &[&[u8]],
    blocks: &[THasDictIndex],
    write_hashes: bool,
) -> Result<DictionarySerializeResult, DictionarySerializeError>
where
    THasDictIndex: HasDictIndex,
{
    serialize_dictionary_payload_with_allocator(dictionaries, blocks, Global, write_hashes)
}

/// Deserializes the dictionary data from its binary format.
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
pub unsafe fn deserialize_dictionary_data(
    dictionary_data: &[u8],
) -> Result<DictionaryData, DictionaryReadError> {
    deserialize_dictionary_data_with_allocator(dictionary_data, Global)
}

/// Deserializes the dictionary data from its binary format using a specified allocator.
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
pub unsafe fn deserialize_dictionary_data_with_allocator<ShortAlloc>(
    dictionary_data: &[u8],
    short_alloc: ShortAlloc,
) -> Result<DictionaryData, DictionaryReadError>
where
    ShortAlloc: Allocator + Clone,
{
    let extracted = extract_payload_with_allocator(dictionary_data, short_alloc.clone())?;
    parse_payload_with_allocator(&extracted)
}

/// Test & bench only method.
#[allow(dead_code)]
pub(crate) fn serialize_dictionary_data<THasDictIndex>(
    dictionaries: &[&[u8]],
    blocks: &[THasDictIndex],
    write_hashes: bool,
) -> Result<Vec<u8>, DictionarySerializeError>
where
    THasDictIndex: HasDictIndex,
{
    // Serialize the dictionary payload
    let payload_result = serialize_dictionary_payload(dictionaries, blocks, write_hashes)?;

    // Calculate the total size: header size + payload size
    let total_size = DictionariesHeader::SIZE_BYTES + payload_result.payload.len();

    // Create a vector to hold the entire serialized data
    let mut data = Vec::with_capacity(total_size);

    // Write the header
    data.extend_from_slice(&payload_result.dict_header.0.to_le_bytes());

    // Write the payload
    data.extend_from_slice(&payload_result.payload);

    Ok(data)
}
