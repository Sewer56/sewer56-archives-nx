/// Used for items that can specify a file size.
pub trait HasFileSize {
    /// Returns the file size of the item in bytes.
    fn file_size(&self) -> u64;
}
