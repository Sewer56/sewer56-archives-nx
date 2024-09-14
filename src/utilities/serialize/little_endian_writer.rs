use core::mem::size_of;
use core::ptr::{copy_nonoverlapping, write_unaligned};

/// A trait for types that can be written in little-endian format.
///
/// Types implementing this trait can be written to a [LittleEndianWriter]
/// both at the current position and at a specified offset.
pub trait WriteAsLittleEndian {
    /// Writes the value in little-endian format at the current position.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it writes directly to memory without bounds checking.
    /// The caller must ensure that the writer has enough space to write the value.
    unsafe fn write_le(self, writer: &mut LittleEndianWriter);

    /// Writes the value in little-endian format at the specified offset.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it writes directly to memory without bounds checking.
    /// The caller must ensure that the writer has enough space to write the value at the given offset.
    ///
    /// # Parameters
    ///
    /// * `writer`: The [LittleEndianWriter] to write to.
    /// * `offset`: The offset in number of bytes from the current position.
    unsafe fn write_at_offset_le(self, writer: &mut LittleEndianWriter, offset_in_bytes: isize);
}

/// A utility for writing data in little-endian format to a raw pointer.
#[derive(Debug)]
pub struct LittleEndianWriter {
    pub(crate) ptr: *mut u8,
}

#[coverage(off)]
impl LittleEndianWriter {
    /// Creates a new [LittleEndianWriter] with the given raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided pointer is valid and points to enough
    /// allocated memory for the intended write operations.
    ///
    /// # Parameters
    ///
    /// * `ptr`: A raw mutable pointer to the memory location where data will be written.
    pub unsafe fn new(ptr: *mut u8) -> Self {
        LittleEndianWriter { ptr }
    }

    /// Writes a value to the current position and advances the pointer.
    ///
    /// This method can write any type that implements the `WriteLittleEndian` trait.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it writes directly to memory without bounds checking.
    /// The caller must ensure that there's enough space to write the value.
    ///
    /// # Parameters
    ///
    /// * `value`: The value to be written in little-endian format.
    #[inline(always)]
    pub unsafe fn write<T: WriteAsLittleEndian>(&mut self, value: T) {
        value.write_le(self);
    }

    /// Writes a value at the specified offset without advancing the pointer.
    ///
    /// This method can write any type that implements the `WriteLittleEndian` trait.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it writes directly to memory without bounds checking.
    /// The caller must ensure that there's enough space to write the value at the given offset.
    ///
    /// # Parameters
    ///
    /// * `value`: The value to be written in little-endian format.
    /// * `offset`: The offset in number of bytes from the current position.
    #[inline(always)]
    pub unsafe fn write_at_offset<T: WriteAsLittleEndian>(
        &mut self,
        value: T,
        offset_in_bytes: isize,
    ) {
        value.write_at_offset_le(self, offset_in_bytes);
    }

    /// Writes a byte slice to the current position and advances the pointer.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it writes directly to memory without bounds checking.
    /// The caller must ensure that there's enough space to write all the bytes in the slice.
    ///
    /// # Parameters
    ///
    /// * `data`: A slice of bytes to be written.
    #[inline(always)]
    pub unsafe fn write_bytes(&mut self, data: &[u8]) {
        copy_nonoverlapping(data.as_ptr(), self.ptr, data.len());
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

// Implement WriteLittleEndian for various integer types
macro_rules! impl_write_little_endian {
    ($($t:ty),*) => {
        $(
            impl WriteAsLittleEndian for $t {
                #[inline(always)]
                #[allow(clippy::size_of_in_element_count)]
                unsafe fn write_le(self, writer: &mut LittleEndianWriter) {
                    write_unaligned(writer.ptr as *mut $t, self.to_le());
                    writer.ptr = writer.ptr.add(size_of::<$t>());
                }

                #[inline(always)]
                unsafe fn write_at_offset_le(self, writer: &mut LittleEndianWriter, offset_in_bytes: isize) {
                    write_unaligned((writer.ptr.offset(offset_in_bytes) as *mut $t), self.to_le());
                }
            }
        )*
    };
}

impl_write_little_endian!(i8, u8, i16, u16, i32, u32, i64, u64);

// Special implementation for floating-point types
macro_rules! impl_write_little_endian_float {
    ($($t:ty),*) => {
        $(
            impl WriteAsLittleEndian for $t {
                #[inline(always)]
                unsafe fn write_le(self, writer: &mut LittleEndianWriter) {
                    copy_nonoverlapping(self.to_le_bytes().as_ptr(), writer.ptr, size_of::<$t>());
                    writer.ptr = writer.ptr.add(size_of::<$t>());
                }

                #[inline(always)]
                unsafe fn write_at_offset_le(self, writer: &mut LittleEndianWriter, offset_in_bytes: isize) {
                    copy_nonoverlapping(
                        self.to_le_bytes().as_ptr(),
                        writer.ptr.offset(offset_in_bytes),
                        size_of::<$t>()
                    );
                }
            }
        )*
    };
}

impl_write_little_endian_float!(f32, f64);
