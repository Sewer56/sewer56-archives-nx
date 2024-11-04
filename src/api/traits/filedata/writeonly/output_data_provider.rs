use crate::{api::traits::*, headers::managed::FileEntry};
use alloc::boxed::Box;
use derive_new::new;
use o2o::o2o;

/// An interface for creating output [`ReadOnlyFileData`] instances.
/// Used for providing write access to data, such as unpacking to disk or memory.
///
/// **Note:** Lifetime of [`OutputDataProvider`] instances is managed by the library.
///
/// # Remarks
///
/// There is a 1:1 relationship between an output file/target and its [`OutputDataProvider`],
/// with each provider writing to a single file. This happens only from a single thread, so only [`Send`]
/// is required.
pub trait OutputDataProvider: Send {
    /// The entry this provider is for.
    /// This is a slice of [`FileEntry`] structure, containing only the minimal needed info
    /// needed for extracting the file.
    fn entry(&self) -> SmallFileEntry;

    /// Gets the output file data behind this provider.
    ///
    /// # Arguments
    ///
    /// * `start` - Start offset into the file (in bytes).
    /// * `length` - Length of the data to retrieve (in bytes).
    ///
    /// # Returns
    ///
    /// A boxed [`ReadOnlyFileData`] instance to write decompressed data to.
    ///
    /// # Errors
    ///
    /// Returns a [`FileProviderError`] if the requested range is invalid or if an I/O error occurs.
    fn get_file_data<'a>(
        &'a self,
        start: u64,
        length: u64,
    ) -> Result<Box<dyn ReadWriteFileData + 'a>, FileProviderError>;
}

#[derive(o2o, Default, Clone, Copy, PartialEq, Eq, Hash, Debug, new)]
#[from_owned(FileEntry)]
pub struct SmallFileEntry {
    /// [u32]/[u64] Size of the file after decompression.
    pub decompressed_size: u64,

    /// `u26` Offset of the file inside the decompressed block.
    pub decompressed_block_offset: u32,

    /// `u18` Index of the first block associated with this file.
    pub first_block_index: u32,
}
