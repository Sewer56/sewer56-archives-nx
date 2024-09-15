use crate::{
    api::enums::compression_preference::CompressionPreference,
    headers::managed::{block_size::BlockSize, file_entry::FileEntry},
};
use std::alloc::{Allocator, Global};

/// This contains the shared 'state' used to build the final binary Table of Contents.
///
/// This state is updated by the blocks which write to it during the packing operation.
///
/// # Safety
///
/// This struct uses uninitialized memory. It's the caller's responsibility to ensure
/// that all elements are properly initialized before reading from them. In this case,
/// this is done by the blocks which write to this state.
pub struct TableOfContentsBuilderState<LongAlloc: Allocator + Clone = Global> {
    /// Used formats for compression of each block.
    block_compressions: Box<[CompressionPreference], LongAlloc>,

    /// Individual block sizes in this structure.
    blocks: Box<[BlockSize], LongAlloc>,

    /// Individual file entries.
    entries: Box<[FileEntry], LongAlloc>,
}

impl<LongAlloc: Allocator + Clone> TableOfContentsBuilderState<LongAlloc> {
    /// Creates a new `TableOfContentsBuilderState` with uninitialized boxes.
    ///
    /// # Arguments
    ///
    /// * `block_count` - The number of blocks.
    /// * `entry_count` - The number of file entries.
    /// * `alloc` - The allocator to use.
    ///
    /// # Safety
    ///
    /// This function creates uninitialized memory. It's the caller's responsibility
    /// to initialize all elements before reading from them.
    pub unsafe fn new(block_count: usize, entry_count: usize, alloc: LongAlloc) -> Self {
        Self {
            block_compressions: Box::new_uninit_slice_in(block_count, alloc.clone()).assume_init(),
            blocks: Box::new_uninit_slice_in(block_count, alloc.clone()).assume_init(),
            entries: Box::new_uninit_slice_in(entry_count, alloc).assume_init(),
        }
    }

    // Getter and setter methods for block_compressions

    /// Gets the compression preference for a block.
    ///
    /// # Safety
    ///
    /// Assumes the index is within bounds and the value has been initialized.
    pub unsafe fn get_block_compression_unchecked(&self, index: usize) -> CompressionPreference {
        *self.block_compressions.get_unchecked(index)
    }

    /// Gets the compression preference for a block, with bounds checking.
    ///
    /// # Safety
    ///
    /// Assumes the value at the given index has been initialized.
    pub unsafe fn get_block_compression(&self, index: usize) -> Option<CompressionPreference> {
        self.block_compressions.get(index).copied()
    }

    /// Sets the compression preference for a block.
    ///
    /// # Safety
    ///
    /// Assumes the index is within bounds.
    pub unsafe fn set_block_compression_unchecked(
        &mut self,
        index: usize,
        value: CompressionPreference,
    ) {
        *self.block_compressions.get_unchecked_mut(index) = value;
    }

    /// Sets the compression preference for a block, with bounds checking.
    pub fn set_block_compression(
        &mut self,
        index: usize,
        value: CompressionPreference,
    ) -> Result<(), TocBuilderError> {
        self.block_compressions
            .get_mut(index)
            .map(|x| *x = value)
            .ok_or(TocBuilderError::IndexOutOfBounds)
    }

    // Getter and setter methods for blocks

    /// Gets a block.
    ///
    /// # Safety
    ///
    /// Assumes the index is within bounds and the value has been initialized.
    pub unsafe fn get_block_unchecked(&self, index: usize) -> BlockSize {
        *self.blocks.get_unchecked(index)
    }

    /// Gets a block, with bounds checking.
    ///
    /// # Safety
    ///
    /// Assumes the value at the given index has been initialized.
    pub unsafe fn get_block(&self, index: usize) -> Option<BlockSize> {
        self.blocks.get(index).copied()
    }

    /// Sets a block.
    ///
    /// # Safety
    ///
    /// Assumes the index is within bounds.
    pub unsafe fn set_block_unchecked(&mut self, index: usize, value: BlockSize) {
        *self.blocks.get_unchecked_mut(index) = value;
    }

    /// Sets a block, with bounds checking.
    pub fn set_block(&mut self, index: usize, value: BlockSize) -> Result<(), TocBuilderError> {
        self.blocks
            .get_mut(index)
            .map(|x| *x = value)
            .ok_or(TocBuilderError::IndexOutOfBounds)
    }

    // Getter and setter methods for entries

    /// Gets a file entry.
    ///
    /// # Safety
    ///
    /// Assumes the index is within bounds and the value has been initialized.
    pub unsafe fn get_entry_unchecked(&self, index: usize) -> FileEntry {
        *self.entries.get_unchecked(index)
    }

    /// Gets a file entry, with bounds checking.
    ///
    /// # Safety
    ///
    /// Assumes the value at the given index has been initialized.
    pub unsafe fn get_entry(&self, index: usize) -> Option<FileEntry> {
        self.entries.get(index).copied()
    }

    /// Sets a file entry.
    ///
    /// # Safety
    ///
    /// Assumes the index is within bounds.
    pub unsafe fn set_entry_unchecked(&mut self, index: usize, value: FileEntry) {
        *self.entries.get_unchecked_mut(index) = value;
    }

    /// Sets a file entry, with bounds checking.
    pub fn set_entry(&mut self, index: usize, value: FileEntry) -> Result<(), TocBuilderError> {
        self.entries
            .get_mut(index)
            .map(|x| *x = value)
            .ok_or(TocBuilderError::IndexOutOfBounds)
    }
}

/// Error type for TableOfContentsBuilderState operations
#[derive(Debug, PartialEq, Eq)]
pub enum TocBuilderError {
    IndexOutOfBounds = 0,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::System;

    #[test]
    fn new_creates_correct_sizes() {
        let state = unsafe { TableOfContentsBuilderState::new(10, 20, System) };
        assert_eq!(state.block_compressions.len(), 10);
        assert_eq!(state.blocks.len(), 10);
        assert_eq!(state.entries.len(), 20);
    }

    #[test]
    fn block_compression_getters_and_setters() {
        let mut state = unsafe { TableOfContentsBuilderState::new(1, 1, System) };

        // Test setters
        assert!(state
            .set_block_compression(0, CompressionPreference::Lz4)
            .is_ok());
        assert_eq!(
            state.set_block_compression(1, CompressionPreference::ZStandard),
            Err(TocBuilderError::IndexOutOfBounds)
        );

        // Test getters
        unsafe {
            assert_eq!(
                state.get_block_compression(0),
                Some(CompressionPreference::Lz4)
            );
            assert_eq!(state.get_block_compression(1), None);
        }

        // Test unchecked methods
        unsafe {
            state.set_block_compression_unchecked(0, CompressionPreference::ZStandard);
            assert_eq!(
                state.get_block_compression_unchecked(0),
                CompressionPreference::ZStandard
            );
        }
    }

    #[test]
    fn block_getters_and_setters() {
        let mut state = unsafe { TableOfContentsBuilderState::new(1, 1, System) };
        let block = BlockSize {
            compressed_size: 100,
        };

        // Test setters
        assert!(state.set_block(0, block).is_ok());
        assert_eq!(
            state.set_block(1, block),
            Err(TocBuilderError::IndexOutOfBounds)
        );

        // Test getters
        unsafe {
            assert_eq!(state.get_block(0), Some(block));
            assert_eq!(state.get_block(1), None);
        }

        // Test unchecked methods
        unsafe {
            let new_block = BlockSize {
                compressed_size: 200,
            };
            state.set_block_unchecked(0, new_block);
            assert_eq!(state.get_block_unchecked(0), new_block);
        }
    }

    #[test]
    fn entry_getters_and_setters() {
        let mut state = unsafe { TableOfContentsBuilderState::new(1, 1, System) };
        let entry = FileEntry {
            hash: 123,
            decompressed_size: 456,
            decompressed_block_offset: 789,
            file_path_index: 101112,
            first_block_index: 131415,
        };

        // Test setters
        assert!(state.set_entry(0, entry).is_ok());
        assert_eq!(
            state.set_entry(1, entry),
            Err(TocBuilderError::IndexOutOfBounds)
        );

        // Test getters
        unsafe {
            assert_eq!(state.get_entry(0), Some(entry));
            assert_eq!(state.get_entry(1), None);
        }

        // Test unchecked methods
        unsafe {
            let new_entry = FileEntry {
                hash: 999,
                decompressed_size: 888,
                decompressed_block_offset: 777,
                file_path_index: 666,
                first_block_index: 555,
            };
            state.set_entry_unchecked(0, new_entry);
            assert_eq!(state.get_entry_unchecked(0), new_entry);
        }
    }
}
