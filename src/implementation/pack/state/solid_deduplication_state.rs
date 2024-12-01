use super::pack_state::DeduplicationError;
use crate::headers::types::xxh3sum::{XXH3sum, XXH3sumHashBuilder};
use hashbrown::HashMap;
use std::sync::RwLock;

/// Represents a file entry in the solid archive deduplication system
#[derive(Clone, Debug, Copy)]
pub struct DeduplicatedSolidFile {
    /// The block index of the file
    pub block_index: u32,

    /// Offset into the decompressed block
    pub decompressed_block_offset: u32,
}

/// This represents the shared state used during deduplication for solid archives
#[derive(Default)]
pub struct SolidDeduplicationState {
    /// Contains a mapping of file hashes to pre-assigned block indexes and offsets
    hash_to_solid_file_details: RwLock<HashMap<XXH3sum, DeduplicatedSolidFile, XXH3sumHashBuilder>>,
}

impl SolidDeduplicationState {
    /// Creates a new SolidDeduplicationState
    pub fn new() -> Self {
        Self {
            hash_to_solid_file_details: RwLock::new(HashMap::with_hasher(
                XXH3sumHashBuilder::default(),
            )),
        }
    }

    /// Creates a new [`SolidDeduplicationState`] with specified capacity
    pub fn with_capacity(num_items: usize) -> Self {
        Self {
            hash_to_solid_file_details: RwLock::new(HashMap::with_capacity_and_hasher(
                num_items,
                XXH3sumHashBuilder::default(),
            )),
        }
    }

    /// Ensures the internal hash map has a specific capacity
    pub fn ensure_capacity(&self, num_items: usize) -> Result<(), DeduplicationError> {
        self.hash_to_solid_file_details
            .write()
            .map_err(|_| DeduplicationError::WriteLockError)?
            .reserve(num_items);
        Ok(())
    }

    /// Attempts to find a duplicate file based on its full hash
    pub fn try_find_duplicate_by_full_hash(
        &self,
        full_hash: XXH3sum,
    ) -> Result<Option<DeduplicatedSolidFile>, DeduplicationError> {
        Ok(self
            .hash_to_solid_file_details
            .read()
            .map_err(|_| DeduplicationError::ReadLockError)?
            .get(&full_hash)
            .copied())
    }

    /// Adds a new file hash to the deduplication state
    pub fn add_file_hash(
        &self,
        full_hash: XXH3sum,
        block_index: u32,
        decompressed_offset: u32,
    ) -> Result<(), DeduplicationError> {
        self.hash_to_solid_file_details
            .write()
            .map_err(|_| DeduplicationError::WriteLockError)?
            .insert(
                full_hash,
                DeduplicatedSolidFile {
                    block_index,
                    decompressed_block_offset: decompressed_offset,
                },
            );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_solid_deduplication() {
        let state = SolidDeduplicationState::new();

        state.add_file_hash(123.into(), 1, 500).unwrap();

        let found = state.try_find_duplicate_by_full_hash(123.into()).unwrap();
        assert!(found.is_some());

        let file_details = found.unwrap();
        assert_eq!(file_details.block_index, 1);
        assert_eq!(file_details.decompressed_block_offset, 500);
    }
}
