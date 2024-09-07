/// Represents an item from which a relative path can be extracted.
/// This relative path is relative to the archive's root directory,
/// and is used to determine where the item is stored in the archive.
///
/// This is used as input into the packer.
pub trait HasRelativePath {
    /// The relative path of the item in the archive.
    fn relative_path(&self) -> &str;
}
