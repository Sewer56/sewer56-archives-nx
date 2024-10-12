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
    ///   - **Max SOLID Block Size**: 64 MiB
    ///   - **Max Block Size**: 512 MiB
    ///   - **Max Content Size**: 128 TiB
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
    ///   - **Max Block Count**: 256 thousand (18 bits)
    ///   - **Max SOLID Block Size**: 64 MiB
    ///   - **Max Block Size**: 512 MiB
    ///   - **Max Content Size**: 128 TiB
    ///   - **Max File Size**: 2^64 Bytes
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
    ///   - `u64`: `DecompressedSize`
    ///   - `u26`: `DecompressedBlockOffset`
    ///   - `u20`: `FilePathIndex`
    ///   - `u18`: `FirstBlockIndex`
    V1 = 1,
}
