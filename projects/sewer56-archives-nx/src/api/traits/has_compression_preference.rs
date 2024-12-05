use crate::api::enums::*;

/// Trait for an item which declares a preferred approach for being compressed.
pub trait HasCompressionPreference {
    /// Preferred algorithm to compress the item with.
    fn compression_preference(&self) -> CompressionPreference;
}
