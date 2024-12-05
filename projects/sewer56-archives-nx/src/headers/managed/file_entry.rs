use derive_new::new;
use endian_writer::{EndianReader, EndianWriter, LittleEndianReader, LittleEndianWriter};

use crate::headers::raw::toc::*;

/// Entry for the individual file.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, new)]
pub struct FileEntry {
    /// [u64] Hash of the file described in this entry.
    pub hash: u64,

    /// [u32]/[u64] Size of the file after decompression.
    pub decompressed_size: u64,

    /// `u26` Offset of the file inside the decompressed block.
    pub decompressed_block_offset: u32,

    /// `u20` Index of the file path associated with this file in the StringPool.
    pub file_path_index: u32,

    /// `u18` Index of the first block associated with this file.
    pub first_block_index: u32,
}

impl FileEntry {
    /// Returns true if the file has 1 or more chunks.
    ///
    /// # Arguments
    ///
    /// * `chunk_size_bytes` - Size of single chunk in archive.
    pub fn is_chunked(&self, chunk_size_bytes: u32) -> bool {
        (self.decompressed_size / chunk_size_bytes as u64) >= 1
    }

    /// Calculated via `decompressed_size` divided by Chunk Size.
    ///
    /// # Arguments
    ///
    /// * `chunk_size_bytes` - Size of single chunk in archive.
    pub fn get_chunk_count(&self, chunk_size_bytes: u32) -> u32 {
        // TODO: An optimized version of this with NativeFileHeader
        let mut count = self.decompressed_size / chunk_size_bytes as u64;
        if self.decompressed_size % chunk_size_bytes as u64 != 0 {
            count += 1;
        }
        count as u32
    }

    /// Writes this managed file entry in the format of `NativeFileEntryV0`.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to write to.
    #[inline(always)]
    pub fn write_as_v0(&self, lewriter: &mut LittleEndianWriter) {
        unsafe {
            lewriter.write_u64_at(self.hash, 0);
            lewriter.write_u32_at(self.decompressed_size as u32, 8);
            lewriter.write_u64_at(
                OffsetPathIndexTuple::new(
                    self.decompressed_block_offset,
                    self.file_path_index,
                    self.first_block_index,
                )
                .into_raw(),
                12,
            );
            lewriter.seek(NativeFileEntryV0::SIZE_BYTES as isize);
        }
    }

    /// Writes this managed file entry in the format of `NativeFileEntryV1`.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to write to.
    #[inline(always)]
    pub fn write_as_v1(&self, lewriter: &mut LittleEndianWriter) {
        unsafe {
            lewriter.write_u64_at(self.hash, 0);
            lewriter.write_u64_at(self.decompressed_size, 8);
            lewriter.write_u64_at(
                OffsetPathIndexTuple::new(
                    self.decompressed_block_offset,
                    self.file_path_index,
                    self.first_block_index,
                )
                .into_raw(),
                16,
            );
            lewriter.seek(NativeFileEntryV1::SIZE_BYTES as isize);
        }
    }

    /// Reads this managed file entry from data serialized as `NativeFileEntryV0`.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read from.
    #[inline(always)]
    pub fn from_reader_v0(&mut self, lereader: &mut LittleEndianReader) {
        unsafe {
            self.hash = lereader.read_u64_at(0);
            self.decompressed_size = lereader.read_u32_at(8) as u64;
            let packed = OffsetPathIndexTuple::from_raw(lereader.read_u64_at(12));
            packed.copy_to(self);
            lereader.seek(NativeFileEntryV0::SIZE_BYTES as isize);
        }
    }

    /// Reads this managed file entry from data serialized as `NativeFileEntryV1`.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read from.
    #[inline(always)]
    pub fn from_reader_v1(&mut self, lereader: &mut LittleEndianReader) {
        unsafe {
            self.hash = lereader.read_u64_at(0);
            self.decompressed_size = lereader.read_u64_at(8);
            let packed = OffsetPathIndexTuple::from_raw(lereader.read_u64_at(16));
            packed.copy_to(self);
            lereader.seek(NativeFileEntryV1::SIZE_BYTES as isize);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default values should be zeroed.
    #[test]
    fn file_entry_default() {
        let entry = FileEntry::default();
        assert_eq!(entry.hash, 0);
        assert_eq!(entry.decompressed_size, 0);
        assert_eq!(entry.decompressed_block_offset, 0);
        assert_eq!(entry.file_path_index, 0);
        assert_eq!(entry.first_block_index, 0);
    }

    /// Chunked size should be calculated correctly.
    #[test]
    fn is_chunked() {
        let entry = FileEntry {
            decompressed_size: 1000,
            ..Default::default()
        };

        assert!(entry.is_chunked(500));
        assert!(entry.is_chunked(1000));
        assert!(!entry.is_chunked(1001));
    }

    // Chunk count should be calculated correctly.
    #[test]
    fn chunk_count() {
        let entry = FileEntry {
            decompressed_size: 1000,
            ..Default::default()
        };

        assert_eq!(entry.get_chunk_count(100), 10);
        assert_eq!(entry.get_chunk_count(200), 5);
        assert_eq!(entry.get_chunk_count(300), 4);
        assert_eq!(entry.get_chunk_count(1000), 1);
        assert_eq!(entry.get_chunk_count(1001), 1);
    }

    /// Tests writing of the V0 format at its natural size.
    #[test]
    fn can_write_and_read_v0() {
        let entry = FileEntry {
            hash: u64::MAX,
            decompressed_size: u32::MAX as u64, // Max value for v0
            decompressed_block_offset: (1 << 26) - 1, // Max value for u26
            file_path_index: (1 << 20) - 1,     // Max value for u20
            first_block_index: (1 << 18) - 1,   // Max value for u18
        };

        let mut buffer = vec![0u8; NativeFileEntryV0::SIZE_BYTES];
        let mut writer = unsafe { LittleEndianWriter::new(buffer.as_mut_ptr()) };
        entry.write_as_v0(&mut writer);

        let mut read_entry = FileEntry::default();
        let mut reader = unsafe { LittleEndianReader::new(buffer.as_ptr()) };
        read_entry.from_reader_v0(&mut reader);

        assert_eq!(entry, read_entry);
    }

    /// Tests writing of the V1 format at its natural size.
    #[test]
    fn can_write_and_read_v1() {
        let entry = FileEntry {
            hash: u64::MAX,
            decompressed_size: u64::MAX,              // Max value for v1
            decompressed_block_offset: (1 << 26) - 1, // Max value for u26
            file_path_index: (1 << 20) - 1,           // Max value for u20
            first_block_index: (1 << 18) - 1,         // Max value for u18
        };

        let mut buffer = vec![0u8; NativeFileEntryV1::SIZE_BYTES];
        let mut writer = unsafe { LittleEndianWriter::new(buffer.as_mut_ptr()) };
        entry.write_as_v1(&mut writer);

        let mut read_entry = FileEntry::default();
        let mut reader = unsafe { LittleEndianReader::new(buffer.as_ptr()) };
        read_entry.from_reader_v1(&mut reader);

        assert_eq!(entry, read_entry);
    }

    /// Tests that the decompressed size is correctly read and written for the V1 format.
    #[test]
    fn v0_decompressed_size_limit_correctly_overflows() {
        let entry = FileEntry {
            decompressed_size: u32::MAX as u64 + 1, // Exceeds v0 limit
            ..Default::default()
        };

        let mut buffer = vec![0u8; NativeFileEntryV0::SIZE_BYTES];
        let mut writer = unsafe { LittleEndianWriter::new(buffer.as_mut_ptr()) };
        entry.write_as_v0(&mut writer);

        let mut read_entry = FileEntry::default();
        let mut reader = unsafe { LittleEndianReader::new(buffer.as_ptr()) };
        read_entry.from_reader_v0(&mut reader);

        assert_eq!(read_entry.decompressed_size, 0); // Should be truncated to 0 (overflow)
    }

    /// Tests that the decompressed size is correctly read and written for the V1 format.
    #[test]
    fn v1_decompressed_size_limit_correctly_overflows() {
        let entry = FileEntry {
            decompressed_size: u64::MAX,
            ..Default::default()
        };

        let mut buffer = vec![0u8; NativeFileEntryV1::SIZE_BYTES];
        let mut writer = unsafe { LittleEndianWriter::new(buffer.as_mut_ptr()) };
        entry.write_as_v1(&mut writer);

        let mut read_entry = FileEntry::default();
        let mut reader = unsafe { LittleEndianReader::new(buffer.as_ptr()) };
        read_entry.from_reader_v1(&mut reader);

        assert_eq!(entry.decompressed_size, read_entry.decompressed_size);
    }

    #[test]
    fn offset_path_index_tuple_limits() {
        let entry = FileEntry {
            decompressed_block_offset: (1 << 26) - 1, // Max value for u26
            file_path_index: (1 << 20) - 1,           // Max value for u20
            first_block_index: (1 << 18) - 1,         // Max value for u18
            ..Default::default()
        };

        let mut buffer = vec![0u8; NativeFileEntryV1::SIZE_BYTES];
        let mut writer = unsafe { LittleEndianWriter::new(buffer.as_mut_ptr()) };
        entry.write_as_v1(&mut writer);

        let mut read_entry = FileEntry::default();
        let mut reader = unsafe { LittleEndianReader::new(buffer.as_ptr()) };
        read_entry.from_reader_v1(&mut reader);

        assert_eq!(entry, read_entry);
    }
}
