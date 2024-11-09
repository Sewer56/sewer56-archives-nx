//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![feature(coverage_attribute)]
#![feature(allocator_api)]
#![feature(once_cell_try)]
#[cfg(feature = "c_exports")]
pub mod exports;
extern crate alloc;

/// Public High Level API
pub mod api {
    pub mod enums;

    /// Allows for specifying inputs and outputs for pack and extract operations.
    pub mod filedata;

    /// Public APIs related to packing.
    pub mod packing {
        pub mod packer_file;
        pub mod packing_settings;
    }

    /// This contains traits that are implementable by outside entities
    /// that wish to integrate with the library.
    pub mod traits;
}

/// This module contains all of the data structures that you'll
/// find within the Nx headers.
///
/// This contains both the 'raw' implementations that match what you'll find in the file,
/// and the 'managed' implementations that are more ergonomic to work with.
pub mod headers {
    pub mod enums;

    /// This contains the serialization/deserialization logic for various parts of Table of Contents
    /// with variable sizes.
    pub mod parser;

    /// This module contains all of the raw data structures that match 1:1 what's in the file.
    pub mod raw {
        pub mod native_file_header;
        pub mod toc;
    }

    /// This represents the unpacked 'managed' version of the headers.
    pub mod managed;

    /// This contains reused traits associated with headers.
    pub mod traits {}

    /// Various data types, usually nominally typed.
    pub mod types {
        /// XXH3 checksums
        pub mod xxh3sum;
    }
}

/// This contains the implementation of the low level APIs.
pub mod implementation {
    /// Implementation of the NX packing logic.
    pub mod pack {

        pub mod blocks {
            pub mod polyfills;
        }

        /// Stores the current state of the table of contents as it is
        /// built by the individual blocks.
        pub mod table_of_contents_builder_state;
    }
}

pub mod structs {}

pub mod utilities {

    /// Utilities for grouping, sorting and general arrangement of items.
    pub mod arrange {
        pub mod sort_lexicographically;
        /// Packing related arrangement steps.
        pub mod pack {
            /// Groups the files by extension.
            pub mod group_by_extension;
            /// Creates the blocks from a set of input files.
            pub mod make_blocks;
        }
    }

    /// This module contains APIs that abstract the supported compression algorithms.
    pub mod compression;

    /// Exposes the system information.
    pub mod system_info;

    /// Number related code.
    pub mod math;

    #[cfg(test)]
    pub mod tests {
        pub mod packer_file_for_testing;
        pub mod packing_test_helpers;
        pub mod permutations;
    }
}
