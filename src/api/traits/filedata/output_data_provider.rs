use super::{FileData, FileProviderError};
use crate::headers::managed::FileEntry;
use alloc::boxed::Box;

/// An interface for creating output [`FileData`] instances.
/// Used for providing write access to data, such as unpacking to disk or memory.
///
/// **Note:** Lifetime of [`OutputDataProvider`] instances is managed by the library.
///
/// # Remarks
///
/// There is a 1:1 relationship between an output file/target and its [`OutputDataProvider`],
/// ensuring that each provider instance is accessed by only one thread at a time.
///
/// # Thread Safety
///
/// Implementations of this trait are [`Send`] but do **not** require [`Sync`].
pub trait OutputDataProvider: Send {
    /// The entry this provider is for.
    /// The unpacker uses this to determine how to extract the data.
    ///
    /// # Returns
    ///
    /// A reference to the associated [`FileEntry`].
    fn entry(&self) -> &FileEntry;

    /// Gets the output file data behind this provider.
    ///
    /// # Arguments
    ///
    /// * `start` - Start offset into the file (in bytes).
    /// * `length` - Length of the data to retrieve (in bytes).
    ///
    /// # Returns
    ///
    /// A boxed [`FileData`] instance to write decompressed data to.
    ///
    /// # Errors
    ///
    /// Returns a [`FileProviderError`] if the requested range is invalid or if an I/O error occurs.
    fn get_file_data(
        &self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn FileData + Send>, FileProviderError>;
}
