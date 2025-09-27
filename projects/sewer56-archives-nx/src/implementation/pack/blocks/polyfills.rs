use crate::api::enums::*;
use crate::api::traits::*;
use crate::prelude::*;
use alloc::{rc::Rc, sync::Arc};
use core::any::Any;
use core::slice;
use hashbrown::HashTable;

/// The value that indicates no dictionary is used.
pub const NO_DICTIONARY_INDEX: u8 = u8::MAX;

// Simple table entry that just stores the pointer value
#[derive(Debug)]
pub struct PtrEntry {
    key: u64,
}

// Define the Block trait
pub trait Block<T>: HasDictIndex
where
    T: HasFileSize + CanProvideInputData + HasRelativePath,
{
    // Define necessary methods
    fn as_any(&self) -> &dyn Any;

    /// Appends files to a given vector, using a [`HashTable`] to track duplicates.
    ///
    /// # Arguments
    /// * `items` - The vector to append items to
    /// * `seen` - HashSet tracking already added items
    fn append_items(&self, items: &mut Vec<Rc<T>>, seen: &mut HashTable<PtrEntry>);

    /// Returns a slice of references to all items in this block without allocation
    fn items(&self) -> &[Rc<T>];

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
    T: HasFileSize + CanProvideInputData + HasRelativePath,
{
    pub(crate) items: Vec<Rc<T>>,
    pub(crate) compression_preference: CompressionPreference,
    pub(crate) dict_index: u32,
}

impl<T> SolidBlock<T>
where
    T: HasFileSize + CanProvideInputData + HasRelativePath,
{
    /// Create a new SolidBlock
    ///
    /// # Arguments
    /// * `items` - The items in the block
    /// * `compression_preference` - The preferred compression algorithm
    /// * `dict_index` - The index of the dictionary, if using dictionary compression.
    pub fn new(
        items: Vec<Rc<T>>,
        compression_preference: CompressionPreference,
        dict_index: u32,
    ) -> Self {
        SolidBlock {
            items,
            compression_preference,
            dict_index,
        }
    }
}

impl<T> Block<T> for SolidBlock<T>
where
    T: HasFileSize + CanProvideInputData + HasRelativePath + 'static,
{
    // Implement necessary methods
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn append_items(&self, items: &mut Vec<Rc<T>>, seen: &mut HashTable<PtrEntry>) {
        for item in &self.items {
            let ptr_value = Rc::as_ptr(item) as u64;

            if seen
                .find(ptr_value, |entry| entry.key == ptr_value)
                .is_none()
            {
                seen.insert_unique(ptr_value, PtrEntry { key: ptr_value }, |entry| entry.key);
                items.push(item.clone());
            }
        }
    }

    fn items(&self) -> &[Rc<T>] {
        &self.items
    }

    fn max_decompressed_block_offset(&self) -> u32 {
        0
    }
}

impl<T: HasFileSize + CanProvideInputData + HasRelativePath> HasDictIndex for SolidBlock<T> {
    fn dict_index(&self) -> u32 {
        self.dict_index
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

    /// Index of the dictionary for dictionary compression, if dictionary
    /// compression is being used.
    pub(crate) dict_index: u32,
}

impl<T> ChunkedBlockState<T>
where
    T: HasFileSize + CanProvideInputData + HasRelativePath,
{
    /// Create a new ChunkedBlockState
    ///
    /// # Arguments
    /// * `compression` - The preferred compression algorithm
    /// * `num_chunks` - The number of chunks
    /// * `file` - The file being compressed
    /// * `dict_index` - The index of the dictionary, if using dictionary compression.
    pub fn new(
        compression: CompressionPreference,
        num_chunks: u32,
        file: Rc<T>,
        dict_index: u32,
    ) -> Self {
        Self {
            compression,
            num_chunks,
            file,
            dict_index,
        }
    }
}

/// A block that represents the slice of an existing file.
#[allow(dead_code)]
pub struct ChunkedFileBlock<T>
where
    T: HasFileSize + CanProvideInputData + HasRelativePath,
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
    T: HasFileSize + CanProvideInputData + HasRelativePath,
{
    /// Creates a new ChunkedFileBlock
    ///
    /// # Arguments
    ///
    /// * `start_offset` - The starting offset of the block
    /// * `chunk_size` - The size of the block at the start offset.
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
    T: HasFileSize + CanProvideInputData + HasRelativePath + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn append_items(&self, items: &mut Vec<Rc<T>>, seen: &mut HashTable<PtrEntry>) {
        let ptr_value = Rc::as_ptr(&self.state.file) as u64;

        if seen
            .find(ptr_value, |entry| entry.key == ptr_value)
            .is_none()
        {
            seen.insert_unique(ptr_value, PtrEntry { key: ptr_value }, |entry| entry.key);
            items.push(self.state.file.clone());
        }
    }

    fn items(&self) -> &[Rc<T>] {
        // Create a static slice containing just the single file reference
        slice::from_ref(&self.state.file)
    }
}

impl<T: CanProvideInputData + HasFileSize + HasRelativePath> HasDictIndex for ChunkedFileBlock<T> {
    fn dict_index(&self) -> u32 {
        self.state.dict_index
    }
}

// Blanket implmementation
impl<T> HasDictIndex for Box<dyn Block<T>>
where
    T: HasFileSize + CanProvideInputData + HasRelativePath,
{
    fn dict_index(&self) -> u32 {
        (**self).dict_index()
    }
}
