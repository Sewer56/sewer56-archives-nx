use derive_new::new;
use thiserror_no_std::Error;

use crate::{
    api::enums::compression_preference::CompressionPreference,
    headers::{managed::*, parser::*},
};
use std::alloc::{Allocator, Global};

/// Managed representation of the deserialized table of contents.
/// Used for both NX v1.x.x and NX v2.x.x
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
}

/// Errors that can occur when deserializing TableOfContents
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum DeserializeError {
    /// Error unpacking the string pool
    StringPoolUnpackError(#[from] StringPoolUnpackError),
    /// Unsupported table of contents version
    UnsupportedTocVersion,
    /// Error that occurs when there is insufficient data to deserialize the ToC
    InsufficientData(#[from] InsufficientDataError),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Error, new)]
pub struct InsufficientDataError {
    /// Actual number of available bytes. (less than expected)
    pub available: u32,
    /// Expected minimum number of available bytes.
    pub expected: u32,
}
