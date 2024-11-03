/// An interface for providing read access to a file.
/// From any source, be it memory, network, disk or other.
///
/// # Remarks
///
/// The [`FileData`] has a lifetime limited by the [`InputDataProvider`] instance where it was
/// created from. For more details, see the documentation for [`InputDataProvider`].
///
/// [`InputDataProvider`]: crate::api::traits::filedata::input_data_provider::InputDataProvider
pub trait FileData {
    /// Returns a byte slice of the underlying file data.
    ///
    /// # Returns
    ///
    /// A byte slice representing the underlying data.
    fn data(&self) -> &[u8];
}
