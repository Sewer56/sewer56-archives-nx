/// Represents an individual block in the Table of Contents (ToC)
///
/// # Remarks
///
/// Max (Compressed) Block Size in ToC today is 512MB (`u29`) by definition.
/// Although the main file header can go larger, this is currently
#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockSize {
    /// Compressed size of the block.
    pub compressed_size: u32,
}
