/// An interface for providing write access to an output of the R3A library.
/// From any source, be it memory, network, disk or other.
///
/// # Remarks
///
/// The [`ReadWriteFileData`] has a lifetime limited by the [`OutputDataProvider`] instance where it was
/// created from. For more details, see the documentation for [`OutputDataProvider`].
///
/// [`OutputDataProvider`]: crate::api::traits::filedata::output_data_provider::OutputDataProvider
pub trait ReadWriteFileData {
    /// Returns a byte slice of the underlying file data.
    ///
    /// # Returns
    ///
    /// A byte slice representing the underlying data.
    fn data(&mut self) -> &mut [u8];
}
