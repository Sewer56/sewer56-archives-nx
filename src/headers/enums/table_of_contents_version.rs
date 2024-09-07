use int_enum::IntEnum;

/// Dictates the version/variant of the archive.
/// Range: 0-3
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, IntEnum)]
pub enum TableOfContentsVersion {
    /// 20 byte FileEntry with u32 sizes.
    /// 1 million file limit. Covers 99.9% of the cases.
    V0 = 0,

    /// 24 byte FileEntry with u64 sizes.
    /// 1 million file limit. Covers 99.9% of the cases.
    V1 = 1,
}
