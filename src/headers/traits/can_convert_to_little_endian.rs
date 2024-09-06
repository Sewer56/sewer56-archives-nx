/// Trait for types that can convert their endianness to little-endian.
/// This is a no-op on Little Endian platforms and a conversion on big-endian platforms.
pub trait CanConvertToLittleEndian {
    /// Converts the type to little-endian format.
    fn to_le(&self) -> Self;
}
