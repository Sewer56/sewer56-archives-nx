use super::*;
use nanokit::count_bits::BitsNeeded;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ToCFormat {
    FEF64,
    Preset0,
    Preset1,
    Preset2,
    Preset3,
    Preset3NoHash,
    Error,
}

#[allow(clippy::absurd_extreme_comparisons)] // <= is more understandable than == in context.
pub fn determine_optimal_toc_format(
    string_pool_size: u32,
    max_decompressed_block_offset: u32,
    block_count: u32,
    file_count: u32,
    hashes_required: bool,
    max_file_size: u64,
) -> ToCFormat {
    // In order of preference (by file size first, then decode complexity for those with equal size):
    // - [8B] Preset3 [a.k.a. NoSolid] (no hash)
    // - [8B] FEF64 (no hash)
    // - [12B] Preset1 [a.k.a. NoHash]
    // - [16B] Preset3 [a.k.a. NoSolid] (with hash)
    // - [16B] FEF64 (with hash)
    // - [20B] Preset0 [a.k.a. GeneralFallback]
    // - [24B] Preset2 [a.k.a. FinalFallback]
    let supports_fef64 = can_use_fef64(
        string_pool_size,
        max_decompressed_block_offset,
        block_count,
        file_count,
        max_file_size,
    );

    let supports_preset_3 = string_pool_size <= PRESET3_STRING_POOL_SIZE_MAX
        && block_count <= PRESET3_BLOCK_COUNT_MAX
        && file_count <= PRESET3_FILE_COUNT_MAX
        && max_decompressed_block_offset <= PRESET3_MAX_DECOMPRESSED_BLOCK_OFFSET
        && max_file_size <= PRESET3_MAX_FILE_SIZE as u64;

    // 0. Check Preset3 (no hash)
    if supports_preset_3 && !hashes_required {
        return ToCFormat::Preset3NoHash;
    }

    // 1. Check FEF64 (no hash)
    if !hashes_required && supports_fef64 {
        return ToCFormat::FEF64;
    }

    // 2. Check Preset1 [Nohash]
    if !hashes_required
        && string_pool_size <= PRESET1_STRING_POOL_SIZE_MAX
        && block_count <= PRESET1_BLOCK_COUNT_MAX
        && file_count <= PRESET1_FILE_COUNT_MAX
        && max_file_size <= PRESET1_MAX_FILE_SIZE as u64
    {
        return ToCFormat::Preset1;
    }

    // 3. Check Preset3 (with hash)
    if supports_preset_3 && hashes_required {
        return ToCFormat::Preset3;
    }

    // 4. Check FEF64
    if supports_fef64 {
        return ToCFormat::FEF64;
    }

    // 5. Check Preset0 [first fallback]
    if hashes_required
        && string_pool_size <= PRESET0_STRING_POOL_SIZE_MAX
        && block_count <= PRESET0_BLOCK_COUNT_MAX
        && file_count <= PRESET0_FILE_COUNT_MAX
        && max_decompressed_block_offset <= PRESET0_DECOMPRESSED_BLOCK_OFFSET_MAX
        && max_file_size <= PRESET0_MAX_FILE_SIZE as u64
    {
        return ToCFormat::Preset0;
    }

    // 6. Check Preset2 [final fallback]
    if hashes_required
        && string_pool_size <= PRESET2_STRING_POOL_SIZE_MAX
        && block_count <= PRESET2_BLOCK_COUNT_MAX
        && file_count <= PRESET2_FILE_COUNT_MAX
        && max_file_size <= PRESET2_MAX_FILE_SIZE
    {
        return ToCFormat::Preset2;
    }

    // 7. Fallback to Error
    ToCFormat::Error
}

// Helper function to determine if FEF64 can be used
pub(crate) fn can_use_fef64(
    string_pool_size: u32,
    max_decompressed_block_offset: u32,
    block_count: u32,
    file_count: u32,
    max_file_size: u64,
) -> bool {
    // Calculate the number of bits needed for each field
    let bits_needed_string_pool_size = string_pool_size.bits_needed_to_store();
    let bits_needed_file_count = file_count.bits_needed_to_store();
    let bits_needed_block_count = block_count.bits_needed_to_store();
    let bits_needed_decompressed_block_offset =
        max_decompressed_block_offset.bits_needed_to_store();

    // Ensure that each field's bits do not exceed the 5-bit allocation in the header
    // u5 can represent up to 31 bits
    if bits_needed_string_pool_size > 31
        || bits_needed_file_count > 31
        || bits_needed_block_count > 31
        || bits_needed_decompressed_block_offset > 31
    {
        return false;
    }

    // Calculate bits remaining for DecompressedSize
    let bits_remaining = 64
        - bits_needed_decompressed_block_offset
        - bits_needed_file_count
        - bits_needed_block_count;

    // Calculate bits needed to store max_file_size
    let bits_needed_file_size = max_file_size.bits_needed_to_store();

    // Ensure that the remaining bits can store the max_file_size
    bits_needed_file_size <= bits_remaining
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that Preset3NoHash is selected when:
    /// - `hashes_required` is `false`.
    /// - All Preset3NoHash constraints are satisfied.
    #[test]
    fn will_use_preset3nohash_when_all_preset3nohash_conditions_met() {
        let toc = determine_optimal_toc_format(
            PRESET3_STRING_POOL_SIZE_MAX,
            PRESET3_MAX_DECOMPRESSED_BLOCK_OFFSET,
            PRESET3_BLOCK_COUNT_MAX,
            PRESET3_FILE_COUNT_MAX,
            false,
            PRESET3_MAX_FILE_SIZE as u64,
        );
        assert_eq!(toc, ToCFormat::Preset3NoHash);
    }

    /// Tests that Preset1 is selected when:
    /// - `hashes_required` is `false`.
    /// - All Preset1 constraints are satisfied.
    ///
    /// - Preset3NoHash constraints are not satisfied [we have decompressed block offset]
    #[test]
    fn will_use_preset1_when_nohash_and_preset1_conditions_met() {
        let toc = determine_optimal_toc_format(
            PRESET1_STRING_POOL_SIZE_MAX,
            1000, // Arbitrary value within Preset1 constraints
            PRESET1_BLOCK_COUNT_MAX,
            PRESET1_FILE_COUNT_MAX,
            false,
            PRESET1_MAX_FILE_SIZE as u64,
        );
        assert_eq!(toc, ToCFormat::Preset1);
    }

    /// Tests that Preset3 is selected when:
    /// - `hashes_required` is `true`.
    /// - All Preset3 constraints are satisfied.
    ///
    /// - Preset3NoHash constraints are not satisfied [we have hash]
    /// - Preset1 constraints are not satisfied [we have hash]
    /// - FEF64 is not selected because we're non-SOLID.
    #[test]
    fn will_use_preset3_when_all_preset3_conditions_met() {
        let toc = determine_optimal_toc_format(
            PRESET3_STRING_POOL_SIZE_MAX,
            PRESET3_MAX_DECOMPRESSED_BLOCK_OFFSET,
            PRESET3_BLOCK_COUNT_MAX,
            PRESET3_FILE_COUNT_MAX,
            true,
            PRESET3_MAX_FILE_SIZE as u64,
        );
        assert_eq!(toc, ToCFormat::Preset3);
    }

    /// Tests that FEF64 is selected when:
    /// - `hashes_required` is `true`.
    /// - All FEF64 constraints are satisfied.
    ///
    /// - Preset3NoHash constraints are not satisfied [we have hash]
    /// - Preset1 constraints are not satisfied [we have hash]
    /// - Preset3 constraints are not satisfied [we're SOLID]
    #[test]
    fn will_use_fef64_when_hash_and_block_offset_required() {
        let toc = determine_optimal_toc_format(4096, 1024, 256, 256, true, 1024 * 1024);
        assert_eq!(toc, ToCFormat::FEF64);
    }

    /// Tests that Preset0 is selected when:
    /// - `hashes_required` is `true`.
    /// - All Preset0 constraints are satisfied.
    ///
    /// - Preset3NoHash constraints are not satisfied [we have hash]
    /// - Preset1 constraints are not satisfied [we have hash]
    /// - Preset3 constraints are not satisfied [we're SOLID]
    /// - FEF64 constraints are not satisfied [values out of range]
    #[test]
    fn will_use_preset0_when_hashes_required_and_preset0_conditions_met() {
        let toc = determine_optimal_toc_format(
            PRESET0_STRING_POOL_SIZE_MAX,
            PRESET0_DECOMPRESSED_BLOCK_OFFSET_MAX,
            PRESET0_BLOCK_COUNT_MAX,
            PRESET0_FILE_COUNT_MAX,
            true,
            PRESET0_MAX_FILE_SIZE as u64,
        );
        assert_eq!(toc, ToCFormat::Preset0);
    }

    /// Tests that Preset2 is selected when:
    /// - `hashes_required` is `true`.
    /// - All Preset2 constraints are satisfied.
    ///
    /// - Preset3NoHash constraints are not satisfied [we have hash]
    /// - Preset1 constraints are not satisfied [we have hash]
    /// - Preset3 constraints are not satisfied [we're SOLID]
    /// - FEF64 constraints are not satisfied [values out of range]
    /// - Preset0 constraints are not satisfied [file size too big]
    #[test]
    fn will_use_preset2_when_hashes_required_and_preset2_conditions_met() {
        let toc = determine_optimal_toc_format(
            PRESET2_STRING_POOL_SIZE_MAX,
            0, // Preset2 does not use decompressed_block_offset
            PRESET2_BLOCK_COUNT_MAX,
            PRESET2_FILE_COUNT_MAX,
            true,
            PRESET2_MAX_FILE_SIZE,
        );
        assert_eq!(toc, ToCFormat::Preset2);
    }

    /// Tests that Error is returned when no formats, can accommodate the parameters.
    /// - All preset constraints are exceeded.
    #[test]
    fn will_use_error_when_no_formats_fit() {
        let toc = determine_optimal_toc_format(
            PRESET2_STRING_POOL_SIZE_MAX + 1, // Exceeds Preset2 and others
            1,                                // Exceeds Preset3MaxDecompressedBlockOffset
            PRESET2_BLOCK_COUNT_MAX + 1,      // Exceeds Preset2 and others
            PRESET2_FILE_COUNT_MAX + 1,       // Exceeds Preset2 and others
            true,
            PRESET2_MAX_FILE_SIZE, // Exceeds Preset2 and others
        );
        assert_eq!(toc, ToCFormat::Error);
    }
}
