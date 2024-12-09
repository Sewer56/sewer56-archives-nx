/// Preferred option for compression.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum CompressionPreference {
    /// No preference is specified.
    NoPreference = 255,

    // Note: Values below match their encoding in ToC, so we use 255 as 'none'.
    // Note: Max allowed value is 7 in current implementation due to packing.
    /// Do not compress at all, copy data verbatim.
    Copy = 0,

    /// Compress with ZStandard.
    ZStandard = 1,

    /// Compress with LZ4.
    Lz4 = 2,

    /// Compress with BZip3
    Bzip3 = 3,
}
