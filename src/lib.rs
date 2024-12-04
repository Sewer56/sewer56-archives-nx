//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(feature = "nightly", feature(coverage_attribute))]
#![cfg_attr(feature = "nightly", feature(allocator_api))]
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

    /// Public API for starting a packing operation.
    pub mod packer_builder;
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

        pub mod state {
            /// Stores state belonging to a running packing operation
            pub mod pack_state;

            /// Stores the state used to deduplicate chunked blocks
            pub mod chunked_deduplication_state;

            /// Stores the state used to deduplicate solid blocks
            pub mod solid_deduplication_state;
        }
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

    /// Code related to I/O and disk operations
    pub mod io {
        /// Searches a given directory and converts it to a list of files.
        pub mod file_finder;
    }

    #[cfg(test)]
    pub mod tests {
        pub mod mock_block;
        pub mod packer_file_for_testing;
        pub mod packing_test_helpers;
        pub mod permutations;
    }
}

pub mod prelude;
pub use prelude::*;

#[macro_export]
macro_rules! unsize_box2 {
    ($boxed:expr $(,)?) => {
        {
            #[cfg(feature = "nightly")]
            {
                $boxed
            }
            #[cfg(not(feature = "nightly"))]
            {
                let (ptr, allocator) = ::allocator_api2::boxed::Box::into_raw_with_allocator($boxed);
                // we don't want to allow casting to arbitrary type U, but we do want to allow unsize coercion to happen.
                // that's exactly what's happening here -- this is *not* a pointer cast ptr as *mut _, but the compiler
                // *will* allow an unsizing coercion to happen into the `ptr` place, if one is available. And we use _ so that the user can
                // fill in what they want the unsized type to be by annotating the type of the variable this macro will
                // assign its result to.
                let ptr: *mut _ = ptr;
                // SAFETY: see above for why ptr's type can only be something that can be safely coerced.
                // also, ptr just came from a properly allocated box in the same allocator.
                unsafe {
                    ::allocator_api2::boxed::Box::from_raw_in(ptr, allocator)
                }
            }
        }
    }
}
