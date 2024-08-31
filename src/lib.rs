//! # Some Cool Reloaded Library
//! Here's the crate documentation.
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "c-exports")]
pub mod exports;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
