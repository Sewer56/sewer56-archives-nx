use core::mem::size_of;
use core::ptr::{copy_nonoverlapping, read_unaligned};

/// A trait for types that can be read in little-endian format.
///
/// Types implementing this trait can be read from a [LittleEndianReader]
/// both at the current position and at a specified offset.
pub trait ReadAsLittleEndian: Sized {
    /// Reads the value in little-endian format from the current position.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it reads directly from memory without bounds checking.
    /// The caller must ensure that the reader has enough data to read the value.
    unsafe fn read_le(reader: &mut LittleEndianReader) -> Self;

    /// Reads the value in little-endian format from the specified offset.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it reads directly from memory without bounds checking.
    /// The caller must ensure that the reader has enough data to read the value at the given offset.
    ///
    /// # Parameters
    ///
    /// * `reader`: The [LittleEndianReader] to read from.
    /// * `offset`: The offset in number of elements of this type from the current position.
    unsafe fn read_at_offset_le(reader: &mut LittleEndianReader, offset: isize) -> Self;
}

/// A utility for reading data in little-endian format from a raw pointer.
#[derive(Debug)]
pub struct LittleEndianReader {
    ptr: *const u8,
}

#[coverage(off)]
impl LittleEndianReader {
    /// Creates a new [LittleEndianReader] with the given raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided pointer is valid and points to enough
    /// allocated memory for the intended read operations.
    ///
    /// # Parameters
    ///
    /// * `ptr`: A raw const pointer to the memory location from where data will be read.
    pub unsafe fn new(ptr: *const u8) -> Self {
        LittleEndianReader { ptr }
    }

    /// Reads a value from the current position and advances the pointer.
    ///
    /// This method can read any type that implements the `ReadLittleEndian` trait.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it reads directly from memory without bounds checking.
    /// The caller must ensure that there's enough data to read the value.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of value to read, which must implement `ReadLittleEndian`.
    ///
    /// # Returns
    ///
    /// The value read from memory, interpreted in little-endian format.
    #[inline(always)]
    pub unsafe fn read<T: ReadAsLittleEndian>(&mut self) -> T {
        T::read_le(self)
    }

    /// Reads a value at the specified offset without advancing the pointer.
    ///
    /// This method can read any type that implements the `ReadLittleEndian` trait.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it reads directly from memory without bounds checking.
    /// The caller must ensure that there's enough data to read the value at the given offset.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of value to read, which must implement `ReadLittleEndian`.
    ///
    /// # Parameters
    ///
    /// * `offset`: The offset in number of elements of type T from the current position.
    ///
    /// # Returns
    ///
    /// The value read from memory at the specified offset, interpreted in little-endian format.
    #[inline(always)]
    pub unsafe fn read_at_offset<T: ReadAsLittleEndian>(&mut self, offset: isize) -> T {
        T::read_at_offset_le(self, offset)
    }

    /// Reads a byte slice from the current position and advances the pointer.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it reads directly from memory without bounds checking.
    /// The caller must ensure that there's enough data to read all the bytes into the slice.
    ///
    /// # Parameters
    ///
    /// * `data`: A mutable slice to read the bytes into.
    #[inline(always)]
    pub unsafe fn read_bytes(&mut self, data: &mut [u8]) {
        copy_nonoverlapping(self.ptr, data.as_mut_ptr(), data.len());
        self.ptr = self.ptr.add(data.len());
    }

    /// Advances the internal pointer by the specified offset.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it modifies the internal pointer without bounds checking.
    /// The caller must ensure that the new pointer position is valid.
    ///
    /// # Parameters
    ///
    /// * `offset`: The number of bytes to advance the pointer.
    #[inline(always)]
    pub unsafe fn seek(&mut self, offset: isize) {
        self.ptr = self.ptr.offset(offset);
    }
}

// Implement ReadLittleEndian for various integer types
macro_rules! impl_read_little_endian {
    ($($t:ty),*) => {
        $(
            impl ReadAsLittleEndian for $t {
                #[inline(always)]
                #[allow(clippy::size_of_in_element_count)]
                unsafe fn read_le(reader: &mut LittleEndianReader) -> Self {
                    let value = read_unaligned(reader.ptr as *const $t);
                    reader.ptr = reader.ptr.add(size_of::<$t>());
                    <$t>::from_le(value)
                }

                #[inline(always)]
                unsafe fn read_at_offset_le(reader: &mut LittleEndianReader, offset: isize) -> Self {
                    let value = read_unaligned((reader.ptr as *const $t).offset(offset));
                    <$t>::from_le(value)
                }
            }
        )*
    };
}

impl_read_little_endian!(i8, u8, i16, u16, i32, u32, i64, u64);

// Special implementation for floating-point types
macro_rules! impl_read_little_endian_float {
    ($($t:ty),*) => {
        $(
            impl ReadAsLittleEndian for $t {
                #[inline(always)]
                unsafe fn read_le(reader: &mut LittleEndianReader) -> Self {
                    let mut bytes = [0u8; size_of::<$t>()];
                    copy_nonoverlapping(reader.ptr, bytes.as_mut_ptr(), size_of::<$t>());
                    reader.ptr = reader.ptr.add(size_of::<$t>());
                    <$t>::from_le_bytes(bytes)
                }

                #[inline(always)]
                unsafe fn read_at_offset_le(reader: &mut LittleEndianReader, offset: isize) -> Self {
                    let mut bytes = [0u8; size_of::<$t>()];
                    copy_nonoverlapping(
                        reader.ptr.offset(offset * size_of::<$t>() as isize),
                        bytes.as_mut_ptr(),
                        size_of::<$t>()
                    );
                    <$t>::from_le_bytes(bytes)
                }
            }
        )*
    };
}

impl_read_little_endian_float!(f32, f64);
