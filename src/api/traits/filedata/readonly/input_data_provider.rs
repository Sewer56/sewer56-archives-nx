use super::*;
use crate::prelude::*;

/// An interface for creating [`ReadOnlyFileData`] instances.
/// Used for providing read access to data, such as reading files for packing.
///
/// This provides the data (bytes) for an existing file, with a `start` parameter
/// of `0` corresponding to the start of the file.
///
/// # Thread Safety
///
/// There is a 1:1 relationship between a file and its [`InputDataProvider`].
///
/// An [`InputDataProvider`] may be accessed by multiple threads at any given time.
/// This can happen for example when packing files in multiple chunks, the
/// [`InputDataProvider::get_file_data`] will be called in parallel from each chunk.
/// Hence it is [`Send`] in addition to [`Sync`].
///
/// Every returned value from the [`InputDataProvider::get_file_data`] however has its own
/// life time and does not require thread safety. The only constraint is it cannot outlive
/// the original [`InputDataProvider`] instance; which within the use case of the library,
/// it does not.
///
/// # Blocking Behaviour
///
/// For read operations where the entire file is not yet available (e.g., over a network),
/// the provider should stall until it can provide enough data.
///
/// [`ReadOnlyFileData`]: crate::api::traits::filedata::readonly::read_only_file_data::ReadOnlyFileData
pub trait InputDataProvider: Send + Sync {
    /// Gets the file data behind this provider.
    ///
    /// # Arguments
    ///
    /// * `start` - Start offset into the file (in bytes).
    /// * `length` - Length of the data to retrieve (in bytes).
    ///
    /// # Returns
    ///
    /// A boxed [`ReadOnlyFileData`] instance to access the requested data.
    ///
    /// # Errors
    ///
    /// Returns a [`FileProviderError`] if the requested range is invalid or if an I/O error occurs.
    fn get_file_data<'a>(
        &'a self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn ReadOnlyFileData + 'a>, FileProviderError>;
}
