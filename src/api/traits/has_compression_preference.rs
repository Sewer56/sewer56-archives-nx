use crate::api::enums::compression_preference::CompressionPreference;

/// Trait for an item which declares a preferred approach for being compressed.
pub trait HasCompressionPreference {
    /// Preferred algorithm to compress the item with.
    fn compression_preference(&self) -> CompressionPreference;
}
