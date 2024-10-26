use crate::api::enums::*;
use crate::api::traits::*;
use alloc::vec::Vec;
use alloc::{rc::Rc, sync::Arc};
use core::any::Any;

// Define the Block trait
pub trait Block<T>
where
    T: HasFileSize + CanProvideFileData + HasRelativePath,
{
    // Define necessary methods
    fn as_any(&self) -> &dyn Any;

    /// Appends files to a given vector.
    fn append_items(&self, items: &mut Vec<Rc<T>>);

    /// For any block that's based on existing data in another Nx archive, this returns
    /// the max DecompressedBlockOffset for any existing file entry within the block.
    fn max_decompressed_block_offset(&self) -> u32 {
        0
    }
}

/// Represents an individual SOLID block packed by the Nx library.
#[allow(dead_code)]
pub struct SolidBlock<T>
where
    T: HasFileSize + CanProvideFileData + HasRelativePath,
{
    pub(crate) items: Vec<Rc<T>>,
    pub(crate) compression_preference: CompressionPreference,
}

impl<T> SolidBlock<T>
where
    T: HasFileSize + CanProvideFileData + HasRelativePath,
{
    /// Create a new SolidBlock
    ///
    /// # Arguments
    /// * `items` - The items in the block
    /// * `compression_preference` - The preferred compression algorithm
    pub fn new(items: Vec<Rc<T>>, compression_preference: CompressionPreference) -> Self {
        SolidBlock {
            items,
            compression_preference,
        }
    }
}

impl<T> Block<T> for SolidBlock<T>
where
    T: HasFileSize + CanProvideFileData + HasRelativePath + 'static,
{
    // Implement necessary methods
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn append_items(&self, items: &mut Vec<Rc<T>>) {
        items.extend(self.items.iter().cloned());
    }
}

// Implement ChunkedBlockState
#[derive(Clone)]
#[allow(dead_code)]
pub struct ChunkedBlockState<T> {
    /// Number of total chunks in this chunked block.
    ///
    /// # Remarks
    ///
    /// If this value is [`u32::MAX`], then skip processing all blocks.
    pub(crate) num_chunks: u32,

    /// Compression used by all chunks of this file.
    pub(crate) compression: CompressionPreference,

    /// Provides access to the file that's being compressed by
    /// this chunked item.
    pub(crate) file: Rc<T>,
}

impl<T> ChunkedBlockState<T>
where
    T: HasFileSize + CanProvideFileData + HasRelativePath,
{
    /// Create a new ChunkedBlockState
    ///
    /// # Arguments
    /// * `compression` - The preferred compression algorithm
    /// * `num_chunks` - The number of chunks
    /// * `file` - The file being compressed
    pub fn new(compression: CompressionPreference, num_chunks: u32, file: Rc<T>) -> Self {
        Self {
            compression,
            num_chunks,
            file,
        }
    }
}

/// A block that represents the slice of an existing file.
#[allow(dead_code)]
pub struct ChunkedFileBlock<T>
where
    T: HasFileSize + CanProvideFileData + HasRelativePath,
{
    /// Starting offset of this block within the file.
    pub(crate) start_offset: u64,
    /// Size of the block starting at [`Self::start_offset`].
    pub(crate) chunk_size: u32,
    /// Zero based index of this chunk.
    pub(crate) chunk_index: u32,
    /// Stores the shared state of all chunks.
    pub(crate) state: Arc<ChunkedBlockState<T>>,
}

impl<T> ChunkedFileBlock<T>
where
    T: HasFileSize + CanProvideFileData + HasRelativePath,
{
    /// Creates a new ChunkedFileBlock
    ///
    /// # Arguments
    ///
    /// * `start_offset` - The starting offset of the block
    /// * `chunk_size` - The size of the block at [`Self::start_offset`].
    /// * `chunk_index` - The index of the block
    /// * `state` - The shared state of all chunks
    pub fn new(
        start_offset: u64,
        chunk_size: u32,
        chunk_index: u32,
        state: Arc<ChunkedBlockState<T>>,
    ) -> Self {
        ChunkedFileBlock {
            start_offset,
            chunk_size,
            chunk_index,
            state,
        }
    }
}

impl<T> Block<T> for ChunkedFileBlock<T>
where
    T: HasFileSize + CanProvideFileData + HasRelativePath + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn append_items(&self, items: &mut Vec<Rc<T>>) {
        items.push(self.state.file.clone());
    }
}
