use super::file_entry::FileEntry;
use crate::headers::raw::toc::*;
use endian_writer::{EndianWriter, LittleEndianWriter};

pub(crate) fn write_entries_as_v0(lewriter: &mut LittleEndianWriter, entries: &[FileEntry]) {
    let mut index = 0;

    #[cfg(feature = "aggressive_unrolling")]
    {
        let ptr = entries.as_ptr(); // Get a raw pointer to the first element

        // Process the entries in chunks of 2 using unrolling
        // SAFETY: We know that index + 2 <= entries.len()
        // Process the entries in chunks of 2 using unrolling
        while index + 2 <= entries.len() {
            unsafe {
                // Get raw references to two entries at a time using pointer arithmetic
                let first_entry = &*ptr.add(index);
                let second_entry = &*ptr.add(index + 1);

                // Call `write_two_as_v0` with two separate parameters
                write_two_as_v0(lewriter, first_entry, second_entry);
            }
            index += 2;
        }
    }

    // Write any remaining entries (fewer than 2)
    while index < entries.len() {
        entries[index].write_as_v0(lewriter);
        index += 1;
    }
}

/// Writes two [FileEntry] structs in the format of [NativeFileEntryV0].
///
/// This function writes two entries in one batch without looping. It adjusts offsets
/// for each entry and performs a single seek at the end.
///
/// # Arguments
///
/// * `writer` - The writer to write the entries to.
/// * `first_entry` - The first `FileEntry` to write.
/// * `second_entry` - The second `FileEntry` to write.
#[inline(always)]
pub(crate) fn write_two_as_v0(
    lewriter: &mut LittleEndianWriter,
    first_entry: &FileEntry,
    second_entry: &FileEntry,
) {
    unsafe {
        // First entry
        lewriter.write_u64_at(first_entry.hash, 0);
        lewriter.write_u32_at(first_entry.decompressed_size as u32, 8);
        lewriter.write_u64_at(
            OffsetPathIndexTuple::new(
                first_entry.decompressed_block_offset,
                first_entry.file_path_index,
                first_entry.first_block_index,
            )
            .into_raw(),
            12,
        );

        // Second entry (adjusted offset by size of one entry)
        lewriter.write_u64_at(second_entry.hash, NativeFileEntryV0::SIZE_BYTES as isize);
        lewriter.write_u32_at(
            second_entry.decompressed_size as u32,
            NativeFileEntryV0::SIZE_BYTES as isize + 8,
        );
        lewriter.write_u64_at(
            OffsetPathIndexTuple::new(
                second_entry.decompressed_block_offset,
                second_entry.file_path_index,
                second_entry.first_block_index,
            )
            .into_raw(),
            NativeFileEntryV0::SIZE_BYTES as isize + 12,
        );

        // Seek forward by the total size of two entries
        lewriter.seek((2 * NativeFileEntryV0::SIZE_BYTES) as isize);
    }
}

pub(crate) fn write_entries_as_v1(writer: &mut LittleEndianWriter, entries: &[FileEntry]) {
    let mut index: usize = 0;

    // Process the entries in chunks of 2
    // SAFETY: We know that index + 2 <= entries.len()
    #[cfg(feature = "aggressive_unrolling")]
    {
        let ptr = entries.as_ptr(); // Get a raw pointer to the first element

        // Process the entries in chunks of 2 using unrolling
        while index + 2 <= entries.len() {
            unsafe {
                // Get raw references to two entries at a time using pointer arithmetic
                let first_entry = &*ptr.add(index);
                let second_entry = &*ptr.add(index + 1);

                // Call `write_two_as_v1` with two separate parameters
                write_two_as_v1(writer, first_entry, second_entry);
            }
            index += 2;
        }
    }

    // Write any remaining entries (fewer than 2)
    while index < entries.len() {
        entries[index].write_as_v1(writer);
        index += 1;
    }
}

/// Writes two [FileEntry] structs in the format of [NativeFileEntryV1].
///
/// This function writes two entries in one batch without looping. It adjusts offsets
/// for each entry and performs a single seek at the end.
///
/// # Arguments
///
/// * `writer` - The writer to write the entries to.
/// * `first_entry` - The first `FileEntry` to write.
/// * `second_entry` - The second `FileEntry` to write.
#[inline(always)]
pub(crate) fn write_two_as_v1(
    lewriter: &mut LittleEndianWriter,
    first_entry: &FileEntry,
    second_entry: &FileEntry,
) {
    unsafe {
        // First entry
        lewriter.write_u64_at(first_entry.hash, 0);
        lewriter.write_u64_at(first_entry.decompressed_size, 8);
        lewriter.write_u64_at(
            OffsetPathIndexTuple::new(
                first_entry.decompressed_block_offset,
                first_entry.file_path_index,
                first_entry.first_block_index,
            )
            .into_raw(),
            16,
        );

        // Second entry (adjusted offset by size of one entry)
        lewriter.write_u64_at(second_entry.hash, NativeFileEntryV1::SIZE_BYTES as isize);
        lewriter.write_u64_at(
            second_entry.decompressed_size,
            NativeFileEntryV1::SIZE_BYTES as isize + 8,
        );
        lewriter.write_u64_at(
            OffsetPathIndexTuple::new(
                second_entry.decompressed_block_offset,
                second_entry.file_path_index,
                second_entry.first_block_index,
            )
            .into_raw(),
            NativeFileEntryV1::SIZE_BYTES as isize + 16,
        );

        // Seek forward by the total size of two entries
        lewriter.seek((2 * NativeFileEntryV1::SIZE_BYTES) as isize);
    }
}
