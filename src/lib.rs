//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![feature(coverage_attribute)]
#![feature(allocator_api)]
#[cfg(feature = "c-exports")]
pub mod exports;
extern crate alloc;

/// Public High Level API
pub mod api {
    pub mod enums {
        /// Allows you to specify how the data should be compressed.
        pub mod compression_preference;
        /// Allows you to specify whether a given file should be SOLID or not.
        pub mod solid_preference;
    }

    /// Public APIs related to packing.
    pub mod packing {
        pub mod packing_settings;
    }

    /// This contains traits that are implementable by outside entities
    /// that wish to integrate with the library.
    pub mod traits {
        /// Trait for items which can provide bytes corresponding to a file.
        pub mod can_provide_file_data;
        /// Used for items to with which format they would like to be compressed.
        pub mod has_compression_preference;
        /// Indicates the item has a file size. For data input into the packer.
        pub mod has_file_size;
        /// Indicates an item has a relative path. For data input into the packer.
        pub mod has_relative_path;
        /// Used for items to specify a preference on whether they'd prefer to be SOLIDly packed or not.
        pub mod has_solid_type;
    }
}

/// This module contains all of the data structures that you'll
/// find within the Nx headers.
///
/// This contains both the 'raw' implementations that match what you'll find in the file,
/// and the 'managed' implementations that are more ergonomic to work with.
pub mod headers {
    pub mod enums {
        pub mod table_of_contents_version;
    }

    /// This contains the serialization/deserialization logic for various parts of Table of Contents
    /// with variable sizes.
    pub mod parser {
        /// Contains the implementation of the NX string pool.
        pub mod string_pool;

        /// Logic belonging to multiple versions of the string pool.
        pub mod string_pool_common;
    }

    /// This module contains all of the raw data structures that match 1:1 what's in the file.
    pub mod raw {
        pub mod common {
            pub mod offset_index_path_tuple;
        }

        pub mod native_file_entry;
        pub mod native_file_entry_v0;
        pub mod native_file_entry_v1;
        pub mod native_file_header;
        pub mod native_toc_block_entry;
        pub mod native_toc_header;
    }

    /// This represents the unpacked 'managed' version of the headers.
    pub mod managed {
        /// Represents the size of a compressed block following the header.
        pub mod block_size;
        /// Represents a file entry that was decoded from the Table of Contents.
        pub mod file_entry;
        /// Optimized functionality for dealing with file entries.
        pub mod file_entry_intrinsics;
        /// Allows for serialization of the Table of Contents during the packing operation.
        pub mod table_of_contents_builder;
        /// Allows for deserialization of the Table of Contents during the unpacking operation.
        pub mod table_of_contents_reader;
    }

    /// This contains reused traits in the Nx source tree.
    pub mod traits {
        pub mod can_convert_to_little_endian;
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

    pub mod serialize {
        /// This module contains utilities for reading unaligned data via pointer in little-endian format.
        pub mod little_endian_reader;

        /// This module contains utilities for writing unaligned data via pointer in little-endian format.
        pub mod little_endian_writer;
    }

    /// Exposes the system information.
    pub mod system_info;

    #[cfg(test)]
    pub mod tests {
        pub mod packer_file_for_testing;
        pub mod packing_test_helpers;
        pub mod permutations;
    }
}
