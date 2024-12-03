use super::pack_state::DeduplicationError;
use crate::headers::types::xxh3sum::{XXH3sum, XXH3sumHashBuilder};
use hashbrown::{HashMap, HashSet};
use std::sync::RwLock;

/// Represents the index of the first block in a chunked file
#[derive(Clone, Debug, Copy)]
pub struct FirstChunkedBlockIndex(pub u32);

/// This represents the shared state used during deduplication of chunked blocks
#[derive(Default)]
pub struct ChunkedDeduplicationState {
    /// Contains a mapping of file hashes to pre-assigned block indexes
    hash_to_chunked_file_details:
        RwLock<HashMap<XXH3sum, FirstChunkedBlockIndex, XXH3sumHashBuilder>>,

    /// Contains a set of short hashes (typically of first 4096 bytes) that have been seen
    short_hash_set: RwLock<HashSet<XXH3sum, XXH3sumHashBuilder>>,
}

impl ChunkedDeduplicationState {
    /// Creates a new [`ChunkedDeduplicationState`]
    pub fn new() -> Self {
        Self {
            hash_to_chunked_file_details: RwLock::new(HashMap::with_hasher(
                XXH3sumHashBuilder::default(),
            )),
            short_hash_set: RwLock::new(HashSet::with_hasher(XXH3sumHashBuilder::default())),
        }
    }

    /// Creates a new [`ChunkedDeduplicationState`] with specified capacity
    pub fn with_capacity(num_items: usize) -> Self {
        Self {
            hash_to_chunked_file_details: RwLock::new(HashMap::with_capacity_and_hasher(
                num_items,
                XXH3sumHashBuilder::default(),
            )),
            short_hash_set: RwLock::new(HashSet::with_capacity_and_hasher(
                num_items,
                XXH3sumHashBuilder::default(),
            )),
        }
    }

    /// Ensures the internal hashmaps have a specific capacity
    pub fn ensure_capacity(&self, num_items: usize) -> Result<(), DeduplicationError> {
        self.short_hash_set
            .write()
            .map_err(|_| DeduplicationError::WriteLockError)?
            .reserve(num_items);
        self.hash_to_chunked_file_details
            .write()
            .map_err(|_| DeduplicationError::WriteLockError)?
            .reserve(num_items);
        Ok(())
    }

    /// Checks if the hash of the first up to 4096 bytes has been seen before
    pub fn has_potential_duplicate(&self, hash_4096: XXH3sum) -> Result<bool, DeduplicationError> {
        Ok(self
            .short_hash_set
            .read()
            .map_err(|_| DeduplicationError::ReadLockError)?
            .contains(&hash_4096))
    }

    /// Attempts to find a duplicate file based on its full hash
    pub fn try_find_duplicate_by_full_hash(
        &self,
        full_hash: XXH3sum,
    ) -> Result<Option<FirstChunkedBlockIndex>, DeduplicationError> {
        Ok(self
            .hash_to_chunked_file_details
            .read()
            .map_err(|_| DeduplicationError::ReadLockError)?
            .get(&full_hash)
            .copied())
    }

    /// Adds a new file hash to the deduplication state
    pub fn add_file_hash(
        &self,
        short_hash: XXH3sum,
        full_hash: XXH3sum,
        block_index: u32,
    ) -> Result<(), DeduplicationError> {
        self.short_hash_set
            .write()
            .map_err(|_| DeduplicationError::WriteLockError)?
            .insert(short_hash);
        self.hash_to_chunked_file_details
            .write()
            .map_err(|_| DeduplicationError::WriteLockError)?
            .insert(full_hash, FirstChunkedBlockIndex(block_index));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_deduplication() {
        let state = ChunkedDeduplicationState::new();

        state.add_file_hash(123.into(), 456.into(), 1).unwrap();

        assert!(state.has_potential_duplicate(123.into()).unwrap());
        assert!(!state.has_potential_duplicate(789.into()).unwrap());

        let found = state.try_find_duplicate_by_full_hash(456.into()).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, 1);
    }
}
