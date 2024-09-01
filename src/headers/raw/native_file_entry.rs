use crate::headers::managed::file_entry::FileEntry;

/// Common interface for native file entries, used for copying data.
pub trait NativeFileEntry {
    /// Copy contents of the managed file entry to the native one.
    ///
    /// # Arguments
    ///
    /// * `entry` - Source entry.
    fn copy_from(&mut self, entry: &FileEntry);

    /// Copy contents of the native file entry to the managed one.
    ///
    /// # Arguments
    ///
    /// * `entry` - Receiving entry.
    fn copy_to(&self, entry: &mut FileEntry);
}
