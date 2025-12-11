use endian_writer::*;
use identity_hash::{BuildIdentityHasher, IdentityHashable};
use twox_hash::XxHash3_64;

/// A nominally typed xxHash3 checksum.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct XXH3sum(pub u64);

impl XXH3sum {
    /// Computes the checksum of a slice of bytes.
    pub fn create(input: &[u8]) -> XXH3sum {
        XXH3sum(XxHash3_64::oneshot(input))
    }
}

impl From<u64> for XXH3sum {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<XXH3sum> for u64 {
    fn from(val: XXH3sum) -> Self {
        val.0
    }
}

impl EndianReadableAt for XXH3sum {
    unsafe fn read_at<R: EndianReader>(reader: &mut R, offset: isize) -> Self {
        XXH3sum(reader.read_u64_at(offset))
    }
}

impl EndianWritableAt for XXH3sum {
    unsafe fn write_at<W: EndianWriter>(&self, writer: &mut W, offset: isize) {
        writer.write_u64_at(self.0, offset);
    }
}

impl HasSize for XXH3sum {
    const SIZE: usize = size_of::<XXH3sum>();
}

/// Type alias for HashBuilder using IdentityHasher
pub type XXH3sumHashBuilder = BuildIdentityHasher<XXH3sum>;
impl IdentityHashable for XXH3sum {}
