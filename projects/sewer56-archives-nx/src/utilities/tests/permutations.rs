/// Returns values for bit packing tests.
/// This function returns values from numBits 0 to `max_bits` such that:
///
/// Value 0: 0b1
/// Value 1: 0b11
/// Value 2: 0b111
/// Value 3: 0b1111
/// etc.
///
/// These values are used for testing individual bit packed values do not overlap.
pub fn get_bit_packing_overlap_test_values(max_bits: u32) -> impl Iterator<Item = u64> {
    (0..max_bits).map(|x| (1u64 << (x + 1)) - 1)
}