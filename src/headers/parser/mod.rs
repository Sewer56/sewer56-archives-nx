/// Contains the implementation of the NX string pool.
pub mod string_pool;

/// Logic belonging to multiple versions of the string pool.
pub mod string_pool_common;

/// Logic for serializing dictionaries
pub mod dictionary {
    pub mod dictionary_builder;
    pub mod dictionary_builder_wrappers;
    pub mod dictionary_reader;
}

// Prelude
pub use dictionary::{dictionary_builder::*, dictionary_builder_wrappers::*, dictionary_reader::*};
pub use string_pool::*;
pub use string_pool_common::*;
