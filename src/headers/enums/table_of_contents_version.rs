use int_enum::IntEnum;

/// Represents the version of the Table of Contents (TOC) used in the archive files.
/// Range: 0-3
///
/// Each version corresponds to a specific format and set of limitations.
/// The versions are optimized for different use cases, balancing the size of the TOC
/// against the capabilities needed for different archive sizes and types.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, IntEnum)]
pub enum TableOfContentsVersion {
    /// **Version 0**:
    ///
    /// - **Summary**: 20-byte `FileEntry`. Suitable for 99.9% of mods.
    /// - **Purpose**: General archival/unarchival of larger mods.
    /// - **Limits**:
    ///   - **Max File Count**: 1 million (20 bits)
    ///   - **Max Block Count**: 256 thousand (18 bits)
    ///   - **Max Block Size**: 64 MiB
    ///   - **Max Content Size**: 16,384 TiB
    ///   - **Max File Size**: 4 GiB
    ///
    /// **Format**:
    ///
    /// - **TOC Header** (8 bytes):
    ///   - `u2`: Version (`0`)
    ///   - `u24`: `StringPoolSize`
    ///   - `u18`: `BlockCount`
    ///   - `u20`: `FileCount`
    /// - **FileEntry** (20 bytes):
    ///   - `u64`: `FileHash` (XXH3)
    ///   - `u32`: `DecompressedSize`
    ///   - `u26`: `DecompressedBlockOffset`
    ///   - `u20`: `FilePathIndex`
    ///   - `u18`: `FirstBlockIndex`
    V0 = 0,

    /// **Version 1**:
    ///
    /// - **Summary**: 24-byte `FileEntry`. For truly exceptional edge cases.
    /// - **Purpose**: Handling exceptionally large archives.
    /// - **Limits**:
    ///   - **Max File Count**: 1 million (20 bits)
    ///   - **Max Block Count**: 16,384 billion (34 bits)
    ///   - **Max Block Size**: 64 MiB
    ///   - **Max Content Size**: 1,073,741,824 TiB
    ///   - **Max File Size**: 256 GiB
    ///
    /// **Format**:
    ///
    /// - **TOC Header** (8 bytes):
    ///   - `u2`: Version (`1`)
    ///   - `u24`: `StringPoolSize`
    ///   - `u18`: `BlockCount`
    ///   - `u20`: `FileCount`
    /// - **FileEntry** (24 bytes):
    ///   - `u64`: `FileHash` (XXH3)
    ///   - `u38`: `DecompressedSize`
    ///   - `u26`: `DecompressedBlockOffset`
    ///   - `u20`: `FilePathIndex`
    ///   - `u44`: `FirstBlockIndex`
    V1 = 1,

    /// **Version 2**:
    ///
    /// - **Summary**: 12-byte `FileEntry`. Variant of Version 0, but without `FileHash`,
    ///   reduced file count, and increased block count.
    /// - **Purpose**: Used when file hashes are not needed, such as in read-only virtual filesystems.
    /// - **Limits**:
    ///   - **Max File Count**: 256 thousand (18 bits)
    ///   - **Max Block Count**: 1 million (20 bits)
    ///   - **Max Block Size**: 64 MiB
    ///   - **Max Content Size**: 16,384 TiB
    ///   - **Max File Size**: 4 GiB
    ///
    /// **Format**:
    ///
    /// - **TOC Header** (8 bytes):
    ///   - `u2`: Version (`2`)
    ///   - `u24`: `StringPoolSize`
    ///   - `u20`: `BlockCount`
    ///   - `u18`: `FileCount`
    /// - **FileEntry** (12 bytes):
    ///   - `u32`: `DecompressedSize`
    ///   - `u26`: `DecompressedBlockOffset`
    ///   - `u18`: `FilePathIndex`
    ///   - `u20`: `FirstBlockIndex`
    V2 = 2,

    /// **Version 3**:
    ///
    /// - **Summary**: 16-byte `FileEntry`. Fits most small mods and update packages.
    /// - **Purpose**: Optimized for uploads/downloads to/from the internet.
    /// - **Limits**:
    ///   - **Max File Count**: 255 (8 bits)
    ///   - **Max Block Count**: 255 (8 bits)
    ///   - **Max Block Size**: 1 MiB
    ///   - **Max Content Size**: 255 MiB
    ///   - **Max File Size**: 255 MiB
    ///
    /// **Format**:
    ///
    /// - **TOC Header** (8 bytes):
    ///   - `u2`: Version (`3`)
    ///   - `u29`: `StringPoolSize`
    ///   - `u8`: `BlockCount`
    ///   - `u8`: `FileCount`
    ///   - `u17`: Padding (set to zero)
    /// - **FileEntry** (16 bytes):
    ///   - `u64`: `FileHash` (XXH3)
    ///   - `u28`: `DecompressedSize`
    ///   - `u20`: `DecompressedBlockOffset`
    ///   - `u8`: `FilePathIndex`
    ///   - `u8`: `FirstBlockIndex`
    V3 = 3,
}
