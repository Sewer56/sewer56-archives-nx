/// A nominally typed xxHash3 checksum.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct XXH3sum(pub u64);

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
