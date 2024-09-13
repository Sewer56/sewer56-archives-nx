use crate::{
    api::{
        enums::{compression_preference::CompressionPreference, solid_preference::SolidPreference},
        traits::{
            can_provide_file_data::CanProvideFileData,
            has_compression_preference::HasCompressionPreference, has_file_size::HasFileSize,
            has_relative_path::HasRelativePath, has_solid_type::HasSolidType,
        },
    },
    implementation::pack::blocks::polyfills::{
        Block, ChunkedBlockState, ChunkedFileBlock, SolidBlock,
    },
};
use alloc::rc::Rc;
use alloc::sync::Arc;
use core::mem::take;
use hashbrown::HashMap; // esoteric platform safe

// Define the result struct
pub struct BlocksResult<T> {
    pub blocks: Vec<Box<dyn Block<T>>>,
    pub num_solid_blocks: usize,
    pub num_chunked_blocks: usize,
}

/// This is a step of the .NX packing process that involves creating
/// blocks from groups of files created by `group_files`.
/// (In ascending size order)
///
/// The input is a `HashMap` of file groups, where the key is the file extension.
/// Inside each group is a sorted list of files by size.
///
/// For example, suppose you have the following files in the `.txt` group:
/// - `file1.txt` (1 KiB)
/// - `file2.txt` (2 KiB)
/// - `file3.txt` (4 KiB)
///
/// In this scenario, the `make_blocks` step will create a block of `file1.txt`
/// and `file2.txt`, and another block of `file3.txt`.
/// (Files bigger than block size are compressed in a single block)
///
/// Sizes of individual blocks can be further constrained by 'chunk size'.
/// Suppose you have a file which is 100 KiB in size, and the chunk size is 32 KiB.
///
/// This will create 3 (chunk) blocks of 32 KiB each, and 1 (chunk) block of 4 KiB.
///
/// The Nx packing pipeline typically starts with the following steps:
/// - Sort files ascending by size [`sort_lexicographically`]
/// - Group files by extension [`group_by_extension`]
/// - Make blocks from file groups (this function)
///
/// # Parameters
///
/// - `groups`: A `HashMap` where each key is a file extension, and the value is a list of files with that extension.
/// - `block_size`: The maximum size of a solid block.
/// - `chunk_size`: The size to use when chunking oversized files.
/// - `solid_block_algorithm`: The compression preference for solid blocks.
/// - `chunked_block_algorithm`: The compression preference for chunked blocks.
///
/// # Returns
///
/// A `BlocksResult<T>` containing the list of blocks and counts of solid and chunked blocks.
///
/// # Type Parameters
///
/// - `T`: The type of the file items, which must implement several traits to provide necessary functionality.
///
/// # Constraints on `T`
///
/// `T` must implement:
/// - [`HasFileSize`]`HasFileSize`
/// - [`HasSolidType`]
/// - [`HasCompressionPreference`]
/// - [`CanProvideFileData`]
/// - [`HasRelativePath`]
/// - [`Clone`]
///
/// [`sort_lexicographically`]: crate::utilities::arrange::sort_lexicographically
/// [`group_by_extension`]: crate::utilities::arrange::pack::group_by_extension
pub fn make_blocks<T>(
    groups: HashMap<&str, Vec<Rc<T>>>,
    block_size: u32,
    chunk_size: u32,
    mut solid_block_algorithm: CompressionPreference,
    mut chunked_block_algorithm: CompressionPreference,
) -> BlocksResult<T>
where
    T: HasFileSize
        + HasSolidType
        + HasCompressionPreference
        + CanProvideFileData
        + HasRelativePath
        + Clone
        + 'static,
{
    let mut chunked_blocks: Vec<Box<dyn Block<T>>> = Vec::new();
    let mut solid_blocks: Vec<(u64, Box<dyn Block<T>>)> = Vec::new();
    let mut current_block: Vec<Rc<T>> = Vec::new();
    let mut current_block_size: u64 = 0; // Must be u64 because file sizes can exceed u32

    // Default algorithms if no preference is specified
    if solid_block_algorithm == CompressionPreference::NoPreference {
        solid_block_algorithm = CompressionPreference::Lz4;
    }

    if chunked_block_algorithm == CompressionPreference::NoPreference {
        chunked_block_algorithm = CompressionPreference::ZStandard;
    }

    // Make the blocks
    for (_key, values) in groups {
        for item in values {
            // If the item is too big, it's getting chunked, regardless of preference.
            // We treat items above the block size as chunked, they are 'single chunk' files.
            if item.file_size() > block_size as u64 {
                chunk_item(
                    &item,
                    &mut chunked_blocks,
                    chunk_size,
                    chunked_block_algorithm,
                );
                continue;
            }

            // If the item should not be put in a SOLID block, it
            // will be put in a separate block.
            if item.solid_type() == SolidPreference::NoSolid {
                solid_blocks.push((
                    item.file_size(),
                    Box::new(SolidBlock::new(
                        vec![item.clone()],
                        item.compression_preference(),
                    )),
                ));
                continue;
            }

            // Check if the item fits in the current block
            // SAFETY: Block size is limited to 1GiB (fits in 32-bit range)
            if current_block_size + item.file_size() <= block_size as u64 {
                // [Hot Path] Add item to SOLID block
                current_block.push(item.clone());
                current_block_size += item.file_size();
            } else {
                // [Cold Path] Add the current block if it has any items and start a new block
                if !current_block.is_empty() {
                    let cloned = current_block.clone();
                    solid_blocks.push((
                        current_block_size,
                        Box::new(SolidBlock::new(cloned, solid_block_algorithm)),
                    ));
                    current_block.clear();
                }
                current_block.push(item.clone());
                current_block_size = item.file_size();
            }
        }
    }

    // If we have any items left, make sure to append them
    if !current_block.is_empty() {
        solid_blocks.push((
            current_block_size,
            Box::new(SolidBlock::new(
                take(&mut current_block),
                solid_block_algorithm,
            )),
        ));
    }

    // Sort the SOLID blocks by size in descending order
    // This speeds up packing, by ensuring thread that picks up last block has least work at end of operation.
    solid_blocks.sort_by(|a, b| b.0.cmp(&a.0));

    let num_chunked_blocks = chunked_blocks.len();
    let num_solid_blocks = solid_blocks.len();

    // Note: Chunked blocks cannot be reordered due to their nature of being
    // sequential. However we can sort the solid blocks to improve compression efficiency.
    // Append the solid blocks to the chunked blocks.
    for (_size, block) in solid_blocks {
        chunked_blocks.push(block);
    }

    // The final blocks vector contains the chunked blocks (in their original order)
    // followed by the solid blocks (sorted by size descending)
    BlocksResult {
        blocks: chunked_blocks,
        num_solid_blocks,
        num_chunked_blocks,
    }
}

// Implement the chunk_item function
fn chunk_item<T>(
    item: &Rc<T>,
    blocks: &mut Vec<Box<dyn Block<T>>>,
    chunk_size: u32,
    mut chunked_block_algorithm: CompressionPreference,
) where
    T: HasFileSize
        + HasSolidType
        + HasCompressionPreference
        + CanProvideFileData
        + HasRelativePath
        + Clone
        + 'static,
{
    let size_left = item.file_size();
    let num_iterations = (size_left / chunk_size as u64) as u32;
    let remaining_size = (size_left % chunk_size as u64) as u32;
    let num_chunks = if remaining_size > 0 {
        num_iterations + 1
    } else {
        num_iterations
    };

    // Default algorithm if no preference is specified
    if chunked_block_algorithm == CompressionPreference::NoPreference {
        chunked_block_algorithm = CompressionPreference::ZStandard;
    }

    let state = Arc::new(ChunkedBlockState::new(
        chunked_block_algorithm,
        num_chunks,
        item.clone(),
    ));

    let mut current_offset = 0_u64;
    for x in 0..num_iterations {
        blocks.push(Box::new(ChunkedFileBlock::new(
            current_offset,
            chunk_size,
            x,
            state.clone(),
        )));
        current_offset += chunk_size as u64;
    }

    if remaining_size > 0 {
        blocks.push(Box::new(ChunkedFileBlock::new(
            current_offset,
            remaining_size,
            num_iterations,
            state,
        )));
    }
}
