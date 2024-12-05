use crate::headers::managed::*;
use bitfield::bitfield;

bitfield! {
    /// A tuple consisting of:
    /// - `u26` DecompressedBlockOffset
    /// - `u20` FilePathIndex
    /// - `u18` FirstBlockIndex
    /// Used in INativeFileEntry and friends.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct OffsetPathIndexTuple(u64);
    impl Debug;

    /// `u26` Offset of the file inside the decompressed block.
    pub u32, decompressed_block_offset, set_decompressed_block_offset: 63, 38;

    /// `u20` Index of the file path associated with this file in the StringPool.
    pub u32, file_path_index, set_file_path_index: 37, 18;

    /// `u18` Index of the first block associated with this file.
    pub u32, first_block_index, set_first_block_index: 17, 0;
}

impl OffsetPathIndexTuple {
    /// Method for fast initialization of the tuple.
    ///
    /// # Arguments
    ///
    /// * `decompressed_block_offset` - `u26` Offset of decompressed block.
    /// * `file_path_index` - `u20` Index of file path in string pool.
    /// * `first_block_index` - `u18` Index of first block associated with this file.
    pub fn new(
        decompressed_block_offset: u32,
        file_path_index: u32,
        first_block_index: u32,
    ) -> Self {
        let mut tuple = OffsetPathIndexTuple(0);
        tuple.set_decompressed_block_offset(decompressed_block_offset);
        tuple.set_file_path_index(file_path_index);
        tuple.set_first_block_index(first_block_index);
        tuple
    }

    /// Method for fast initialization of the tuple from raw data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw packed data.
    pub fn from_raw(data: u64) -> Self {
        OffsetPathIndexTuple(data)
    }

    /// Converts the tuple to its raw representation.
    pub fn into_raw(&self) -> u64 {
        self.0
    }

    /// Copy the values of this tuple to a managed [`FileEntry`].
    ///
    /// This was written to avoid a stack spill.
    #[inline(always)]
    pub fn copy_to(&self, entry: &mut FileEntry) {
        entry.decompressed_block_offset = self.decompressed_block_offset();
        entry.file_path_index = self.file_path_index();
        entry.first_block_index = self.first_block_index();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utilities::tests::packing_test_helpers::*;
    use crate::utilities::tests::permutations::*;
    use fake::*;
    use rstest::rstest;

    #[test]
    fn is_correct_size_bytes() {
        assert_eq!(size_of::<OffsetPathIndexTuple>(), 8);
    }

    #[rstest]
    #[cfg_attr(miri, ignore)] // no memory accesses, and too slow to test
    fn can_be_packed_values_dont_overlap() {
        let mut tuple = OffsetPathIndexTuple::default();
        for block_offset in get_bit_packing_overlap_test_values(26) {
            for path_index in get_bit_packing_overlap_test_values(20) {
                for block_index in get_bit_packing_overlap_test_values(18) {
                    // Test all 3 possibilities for overlaps
                    // A & C
                    test_packed_properties(
                        &mut tuple,
                        |t, v| t.set_decompressed_block_offset(v as u32),
                        |t| t.decompressed_block_offset() as u64,
                        |t, v| t.set_first_block_index(v as u32),
                        |t| t.first_block_index() as u64,
                        block_offset,
                        block_index,
                    );

                    // A & B
                    test_packed_properties(
                        &mut tuple,
                        |t, v| t.set_decompressed_block_offset(v as u32),
                        |t| t.decompressed_block_offset() as u64,
                        |t, v| t.set_file_path_index(v as u32),
                        |t| t.file_path_index() as u64,
                        block_offset,
                        path_index,
                    );

                    // B & C
                    test_packed_properties(
                        &mut tuple,
                        |t, v| t.set_file_path_index(v as u32),
                        |t| t.file_path_index() as u64,
                        |t, v| t.set_first_block_index(v as u32),
                        |t| t.first_block_index() as u64,
                        path_index,
                        block_index,
                    );
                }
            }
        }
    }

    #[rstest]
    fn decompressed_block_offset_should_be_26_bits() {
        assert_size_bits(
            &mut OffsetPathIndexTuple::default(),
            |t, v| t.set_decompressed_block_offset(v as u32),
            |t| t.decompressed_block_offset() as u64,
            26,
        );
    }

    #[rstest]
    fn file_path_index_should_be_20_bits() {
        assert_size_bits(
            &mut OffsetPathIndexTuple::default(),
            |t, v| t.set_file_path_index(v as u32),
            |t| t.file_path_index() as u64,
            20,
        );
    }

    #[rstest]
    fn first_block_index_should_be_18_bits() {
        assert_size_bits(
            &mut OffsetPathIndexTuple::default(),
            |t, v| t.set_first_block_index(v as u32),
            |t| t.first_block_index() as u64,
            18,
        );
    }

    impl Dummy<Faker> for OffsetPathIndexTuple {
        fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            let mut tuple = OffsetPathIndexTuple::default();
            tuple.set_decompressed_block_offset(rng.gen_range(0..0x3FFFFFF)); // 26 bits
            tuple.set_file_path_index(rng.gen_range(0..0xFFFFF)); // 20 bits
            tuple.set_first_block_index(rng.gen_range(0..0x3FFFF)); // 18 bits
            tuple
        }
    }
}
