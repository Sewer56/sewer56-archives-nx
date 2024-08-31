//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "c-exports")]
pub mod exports;

/// This module contains all of the data structures that you'll
/// find within the Nx headers.
///
/// This contains both the 'raw' implementations that match what you'll find in the file,
/// and the 'managed' implementations that are more ergonomic to work with.
pub mod headers {
    pub mod enums {
        pub mod table_of_contents_version;
    }

    /// This module contains all of the raw data structures that match 1:1 what's in the file.
    pub mod raw {
        pub mod common {
            pub mod offset_index_path_tuple;
        }
    }
}

pub mod structs {}

pub mod utilities {
    #[cfg(test)]
    pub mod tests {
        pub mod packing_test_helpers;
        pub mod permutations;
    }
}
