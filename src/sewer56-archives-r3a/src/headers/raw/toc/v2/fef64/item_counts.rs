/// Static method to determine if given counts can fit within 42 bits based on their bit allocations.
///
/// # Arguments
///
/// * `string_pool_size_bits` - Bits allocated for String Pool Size.
/// * `block_count_bits` - Bits allocated for Block Count.
/// * `file_count_bits` - Bits allocated for File Count.
///
/// # Returns
///
/// `true` if counts fit within 42 bits; otherwise, `false`.
pub fn can_fit_within_42_bits(
    string_pool_size_bits: u8,
    block_count_bits: u8,
    file_count_bits: u8,
) -> bool {
    let total_bits =
        string_pool_size_bits as u32 + block_count_bits as u32 + file_count_bits as u32;
    total_bits <= 42
}

/// Static method to pack counts into a u64.
/// `file_count` is packed into the least significant bits, followed by `block_count`, then `string_pool_size`.
///
/// # Arguments
///
/// * `string_pool_size` - String Pool Size.
/// * `string_pool_size_bits` - Bits allocated for String Pool Size.
/// * `block_count` - Block Count.
/// * `block_count_bits` - Bits allocated for Block Count.
/// * `file_count` - File Count.
/// * `file_count_bits` - Bits allocated for File Count.
/// * `packed` - Mutable reference to store the packed u64.
pub fn pack_item_counts(
    string_pool_size: u64,
    string_pool_size_bits: u8,
    block_count: u64,
    block_count_bits: u8,
    file_count: u64,
    file_count_bits: u8,
    packed: &mut u64,
) {
    let file_count_mask = (1u64 << file_count_bits) - 1;
    let block_count_mask = (1u64 << block_count_bits) - 1;
    let string_pool_size_mask = (1u64 << string_pool_size_bits) - 1;

    let file_count_part = file_count & file_count_mask;
    let block_count_part = (block_count & block_count_mask) << file_count_bits;
    let string_pool_size_part =
        (string_pool_size & string_pool_size_mask) << (file_count_bits + block_count_bits);

    *packed = file_count_part | block_count_part | string_pool_size_part;
}

/// Static method to unpack counts from a [u64].
///
/// # Arguments
///
/// * `packed` - The packed [u64] containing the item counts.
/// * `string_pool_size_bits` - Bits allocated for String Pool Size.
/// * `block_count_bits` - Bits allocated for Block Count.
/// * `file_count_bits` - Bits allocated for File Count.
///
/// # Returns
///
/// A tuple containing `string_pool_size`, `block_count`, and `file_count`.
pub fn unpack_item_counts(
    packed: u64,
    string_pool_size_bits: u8,
    block_count_bits: u8,
    file_count_bits: u8,
) -> (u64, u64, u64) {
    let file_count_mask = (1u64 << file_count_bits) - 1;
    let block_count_mask = (1u64 << block_count_bits) - 1;
    let string_pool_size_mask = (1u64 << string_pool_size_bits) - 1;

    let file_count = packed & file_count_mask;
    let block_count = (packed >> file_count_bits) & block_count_mask;
    let string_pool_size = (packed >> (file_count_bits + block_count_bits)) & string_pool_size_mask;

    (string_pool_size, block_count, file_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_fit_within_42_bits_when_under_limit() {
        // Sum of bits = 10 + 15 + 16 = 41
        assert!(can_fit_within_42_bits(10, 15, 16));
    }

    #[test]
    fn can_fit_within_42_bits_when_exact_limit() {
        // Sum of bits = 14 + 14 + 14 = 42
        assert!(can_fit_within_42_bits(14, 14, 14));
    }

    #[test]
    fn cannot_fit_within_42_bits_when_over_limit() {
        // Sum of bits = 20 + 15 + 8 = 43
        assert!(!can_fit_within_42_bits(20, 15, 8));
    }

    #[test]
    fn can_pack_item_counts_within_bounds() {
        let file_count = 0b1111; // 15
        let file_count_bits = 4;
        let block_count = 0b1100; // 12
        let block_count_bits = 4;
        let string_pool_size = 0b1010; // 10
        let string_pool_size_bits = 4;
        let mut packed = 0u64;

        pack_item_counts(
            string_pool_size,
            string_pool_size_bits,
            block_count,
            block_count_bits,
            file_count,
            file_count_bits,
            &mut packed,
        );

        // Expected packed value:
        // file_count: 1111 (4 bits) => 1111
        // block_count: 1100 (4 bits) shifted by 4 => 1100 0000
        // string_pool_size: 1010 (4 bits) shifted by 8 => 1010 0000 0000
        // Total: 1010 1100 1111 = 0xACF
        assert_eq!(packed, 0xACF);
    }

    #[test]
    fn can_pack_item_counts_with_max_values() {
        let string_pool_size_bits = 10;
        let block_count_bits = 15;
        let file_count_bits = 17;

        let string_pool_size = 1023; // Max value for 10 bits
        let block_count = 32767; // Max value for 15 bits
        let file_count = 1310971; // Max value for 17 bits

        let mut packed = 0u64;
        pack_item_counts(
            string_pool_size,
            string_pool_size_bits,
            block_count,
            block_count_bits,
            file_count,
            file_count_bits,
            &mut packed,
        );

        // Calculate expected packed value
        let expected = file_count
            | (block_count << file_count_bits)
            | (string_pool_size << (file_count_bits + block_count_bits));

        assert_eq!(packed, expected);
    }

    /// Note: This should never happen in practice.
    #[test]
    fn pack_item_counts_with_overflow_values() {
        let string_pool_size_bits = 8;
        let block_count_bits = 8;
        let file_count_bits = 8;

        let string_pool_size = 0x1FF; // 9 bits, should be masked to 8 bits => 0xFF
        let block_count = 0x200; // 9 bits, should be masked to 8 bits => 0x00
        let file_count = 0x123; // 9 bits, should be masked to 8 bits => 0x23

        let mut packed = 0u64;

        pack_item_counts(
            string_pool_size,
            string_pool_size_bits,
            block_count,
            block_count_bits,
            file_count,
            file_count_bits,
            &mut packed,
        );

        #[allow(clippy::identity_op)]
        // Expected packed value after masking:
        // file_count: 0x23
        // block_count: 0x00 << 8
        // string_pool_size: 0xFF << 16
        let expected = 0x23 | (0x00 << 8) | (0xFF << 16);
        assert_eq!(packed, expected);
    }

    #[test]
    fn can_unpack_item_counts_within_bounds() {
        let string_pool_size_bits = 4;
        let block_count_bits = 4;
        let file_count_bits = 4;

        let packed = 0xACF; // Packed value for example in pack test

        let (string_pool_size, block_count, file_count) = unpack_item_counts(
            packed,
            string_pool_size_bits,
            block_count_bits,
            file_count_bits,
        );

        assert_eq!(string_pool_size, 0b1010); // 10
        assert_eq!(block_count, 0b1100); // 12
        assert_eq!(file_count, 0b1111); // 15
    }

    #[test]
    fn can_unpack_item_counts_with_max_values() {
        let string_pool_size_bits = 10;
        let block_count_bits = 15;
        let file_count_bits = 17;

        let string_pool_size = 1023; // Max value for 10 bits
        let block_count = 32767; // Max value for 15 bits
        let file_count = 131071; // Max value for 17 bits

        let mut packed = 0u64;
        pack_item_counts(
            string_pool_size,
            string_pool_size_bits,
            block_count,
            block_count_bits,
            file_count,
            file_count_bits,
            &mut packed,
        );

        let (unpacked_string_pool_size, unpacked_block_count, unpacked_file_count) =
            unpack_item_counts(
                packed,
                string_pool_size_bits,
                block_count_bits,
                file_count_bits,
            );

        assert_eq!(unpacked_string_pool_size, string_pool_size);
        assert_eq!(unpacked_block_count, block_count);
        assert_eq!(unpacked_file_count, file_count);
    }

    #[test]
    fn unpack_item_counts_with_overflow_values() {
        let string_pool_size_bits = 8;
        let block_count_bits = 8;
        let file_count_bits = 8;

        let string_pool_size = 0x1FF; // 9 bits, should be masked to 8 bits => 0xFF
        let block_count = 0x200; // 9 bits, should be masked to 8 bits => 0x00
        let file_count = 0x123; // 9 bits, should be masked to 8 bits => 0x23

        let mut packed = 0u64;

        pack_item_counts(
            string_pool_size,
            string_pool_size_bits,
            block_count,
            block_count_bits,
            file_count,
            file_count_bits,
            &mut packed,
        );

        let (unpacked_string_pool_size, unpacked_block_count, unpacked_file_count) =
            unpack_item_counts(
                packed,
                string_pool_size_bits,
                block_count_bits,
                file_count_bits,
            );

        assert_eq!(unpacked_string_pool_size, 0xFF);
        assert_eq!(unpacked_block_count, 0x00);
        assert_eq!(unpacked_file_count, 0x23);
    }
}
